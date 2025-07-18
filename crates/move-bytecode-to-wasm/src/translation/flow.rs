use crate::translation::table::TableEntry;
use move_abstract_interpreter::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use move_binary_format::file_format::{Bytecode, CodeUnit};
use relooper::{BranchMode, ShapedBlock};
use std::collections::HashMap;
use walrus::ValType;

#[derive(Debug, Clone)]
pub enum Flow {
    Simple {
        // label: u16,
        stack: Vec<ValType>,
        instructions: Vec<Bytecode>,
        branches: HashMap<u16, BranchMode>,
    },
    Sequence(Vec<Flow>),
    Loop {
        loop_id: u16,
        stack: Vec<ValType>,
        body: Box<Flow>,
    },
    IfElse {
        stack: Vec<ValType>,
        then_body: Box<Flow>,
        else_body: Box<Flow>,
    },
    Empty,
}

// TODO: check how we are building up the types stack
impl Flow {
    pub fn get_stack(&self) -> Vec<ValType> {
        match self {
            Flow::Simple { stack, .. } => stack.clone(),
            // TODO: is concat correct here?
            // concat instructions and then build the types stack!
            Flow::Sequence(blocks) => blocks
                .iter()
                .fold(vec![], |acc, f| [acc, f.get_stack()].concat()),
            Flow::Loop { stack, .. } => stack.clone(),
            Flow::IfElse { stack, .. } => stack.clone(),
            Flow::Empty => vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(&self, Self::Empty)
    }

    pub fn new(code_unit: &CodeUnit, entry: &TableEntry) -> Self {
        // Create the control flow graph from the code unit
        let cfg = VMControlFlowGraph::new(&code_unit.code, &code_unit.jump_tables);

        // Reloop the Control Flow Graph,
        // Emscripten paper, original relooper implementation: https://github.com/emscripten-core/emscripten/blob/main/docs/paper.pdf
        // The one we are using: https://github.com/curiousdannii/if-decompiler/blob/master/relooper/src/lib.rs
        let relooped = {
            let nodes: Vec<(u16, Vec<u16>)> = (&cfg as &dyn ControlFlowGraph)
                .blocks()
                .into_iter()
                .map(|b| (b, cfg.successors(b).to_vec()))
                .collect();
            *relooper::reloop(nodes, 0)
        };

        println!("relooped: {relooped:#?}");

        // Context for each block within the control flow graph
        let blocks_ctx: HashMap<u16, (Vec<Bytecode>, Vec<ValType>)> = (&cfg
            as &dyn ControlFlowGraph)
            .blocks()
            .into_iter()
            .map(|b| {
                let start = cfg.block_start(b);
                let end = cfg.block_end(b) + 1;
                let code = &code_unit.code[start as usize..end as usize];

                let mut stack: Vec<ValType> = vec![];
                // If the block contains a Ret instruction, then set the types stack of this block to the expected return type of the function.
                if code.contains(&Bytecode::Ret) {
                    stack = entry.results.clone();
                }

                (start, (code.to_vec(), stack))
            })
            .collect();

        Self::build(&relooped, &blocks_ctx)
    }

    fn build(
        shaped_block: &ShapedBlock<u16>,
        blocks_ctx: &HashMap<u16, (Vec<Bytecode>, Vec<ValType>)>,
    ) -> Flow {
        match shaped_block {
            ShapedBlock::Simple(simple_block) => {
                let block_ctx = blocks_ctx.get(&simple_block.label).unwrap();
                let b = Flow::Simple {
                    stack: block_ctx.1.clone(),
                    instructions: block_ctx.0.clone(),
                    branches: simple_block
                        .branches
                        .iter()
                        .map(|(k, v)| (*k, *v))
                        .collect(),
                };

                // This are blocks immediately dominated by the current block
                let immediate_blocks = simple_block
                    .immediate
                    .as_ref()
                    .map(|b| Self::build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                // Next block follows the current one, in the graph this represents an edge
                let next_block = simple_block
                    .next
                    .as_ref()
                    .map(|b| Self::build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                // Revisit this part. We are flattening a nested structure into a sequence, is it always correct?
                let mut seq = vec![b];
                if !immediate_blocks.is_empty() {
                    seq.push(immediate_blocks);
                }
                if !next_block.is_empty() {
                    seq.push(next_block);
                }

                if seq.len() == 1 {
                    seq.pop().unwrap()
                } else {
                    Flow::Sequence(seq)
                }
            }
            ShapedBlock::Loop(loop_block) => {
                let inner_block = Self::build(&loop_block.inner, blocks_ctx);

                let loop_flow = Flow::Loop {
                    stack: inner_block.get_stack(),
                    loop_id: loop_block.loop_id,
                    body: Box::new(inner_block),
                };

                // Here too, we put the next block in the sequence if it exists
                if let Some(next_block) = &loop_block.next {
                    let next_flow = Self::build(next_block, blocks_ctx);
                    Flow::Sequence(vec![loop_flow, next_flow])
                } else {
                    loop_flow
                }
            }
            ShapedBlock::Multiple(multiple_block) => {
                // The relooper algorithm generates multiple blocks when a conditional jump is present.
                // Based on observations, these multiple blocks typically have 1 or 2 handled blocks (branches).
                // For instance, a Move function with a multi-branch match statement is transformed into nested multiple blocks.
                // Alternatively, multiple blocks with a single branch can occur when there is no else condition --> while (true) { if (condition) { break } }

                match multiple_block.handled.len() {
                    // If there is a single branch, then instead of creating an if/else flow with an empty arm, we just build the flow from the only handled block.
                    1 => Self::build(&multiple_block.handled[0].inner, blocks_ctx),
                    // If there are two branches, we create an if/else flow with the two handled blocks.
                    2 => {
                        let then_arm = Self::build(&multiple_block.handled[0].inner, blocks_ctx);
                        let else_arm = Self::build(&multiple_block.handled[1].inner, blocks_ctx);

                        let then_stack = then_arm.get_stack();
                        let else_stack = else_arm.get_stack();

                        let stack = if !then_stack.is_empty()
                            && !else_stack.is_empty()
                            && then_stack != else_stack
                        {
                            panic!(
                                "Type stacks of if/else branches must be the same or one must be empty. If types: {:?}, Else types: {:?}",
                                then_stack, else_stack
                            );
                        } else if !then_stack.is_empty() {
                            then_stack
                        } else {
                            else_stack // if both are empty, this returns an empty TypesStack
                        };

                        Flow::IfElse {
                            stack,
                            then_body: Box::new(then_arm),
                            else_body: Box::new(else_arm),
                        }
                    }
                    _ => panic!(
                        "Unsupported MultipleBlock with {} branches",
                        multiple_block.handled.len()
                    ),
                }
            }
        }
    }
}
