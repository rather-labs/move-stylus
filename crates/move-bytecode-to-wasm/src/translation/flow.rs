use crate::CompilationContext;
use crate::MappedFunction;
use crate::translation::TypesStack;
use move_abstract_interpreter::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use move_binary_format::file_format::{Bytecode, CodeUnit};
use relooper::{BranchMode, ShapedBlock};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Flow {
    Simple {
        label: u16,
        types_stack: TypesStack,
        instructions: Vec<Bytecode>,
        branches: HashMap<u16, BranchMode>,
    },
    Sequence(Vec<Flow>),
    Loop {
        loop_id: u16,
        types_stack: TypesStack,
        body: Box<Flow>,
    },
    IfElse {
        types_stack: TypesStack,
        then_body: Box<Flow>,
        else_body: Box<Flow>,
    },
    Empty,
}

// TODO: check how we are building up the types stack
impl Flow {
    pub fn get_types_stack(&self) -> TypesStack {
        match self {
            Flow::Simple { types_stack, .. } => types_stack.clone(),
            // TODO: is concat correct here?
            // concat instructions and then build the types stack!
            Flow::Sequence(blocks) => TypesStack(blocks.iter().fold(vec![], |acc, f| {
                [acc, f.get_types_stack().0.clone()].concat()
            })),
            Flow::Loop { types_stack, .. } => types_stack.clone(),
            Flow::IfElse { types_stack, .. } => types_stack.clone(),
            Flow::Empty => TypesStack::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(&self, Self::Empty)
    }

    pub fn new(
        code_unit: &CodeUnit,
        compilation_ctx: &CompilationContext,
        mapped_function: &MappedFunction,
    ) -> Self {
        // Create the control flow graph from the code unit
        let cfg = VMControlFlowGraph::new(&code_unit.code, &code_unit.jump_tables);

        // Reloop the control flow graph. This transforms the cfg into a structured object
        // The relooped cfg is composed of ShapedBlocks
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
        // This context maps the block's starting index to its corresponding bytecode instructions and the expected types stack after the block's execution
        // TODO: is it okey to calculate the types stack in the cfg instead of the relooped cfg?
        let blocks_ctx: HashMap<u16, (Vec<Bytecode>, TypesStack)> = (&cfg as &dyn ControlFlowGraph)
            .blocks()
            .into_iter()
            .map(|b| {
                let start = cfg.block_start(b);
                let end = cfg.block_end(b) + 1;
                let code = &code_unit.code[start as usize..end as usize];

                let mut ts = TypesStack::new();
                for instruction in code {
                    ts.process_instruction(instruction, compilation_ctx, mapped_function)
                        .unwrap();
                }

                (start, (code.to_vec(), ts))
            })
            .collect();

        let mut flow = Flow::Empty;
        flow.build(&relooped, &blocks_ctx)
    }

    fn build(
        &mut self,
        shaped_block: &ShapedBlock<u16>,
        blocks_ctx: &HashMap<u16, (Vec<Bytecode>, TypesStack)>,
    ) -> Flow {
        match shaped_block {
            ShapedBlock::Simple(simple_block) => {
                let block_ctx = blocks_ctx.get(&simple_block.label).unwrap();
                let b = Flow::Simple {
                    types_stack: block_ctx.1.clone(),
                    label: simple_block.label,
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
                    .map(|b| self.build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                // Next block follows the current one, in the graph this represents an edge
                let next_block = simple_block
                    .next
                    .as_ref()
                    .map(|b| self.build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                // revisit this part
                if !immediate_blocks.is_empty() || !next_block.is_empty() {
                    Flow::Sequence(vec![b, immediate_blocks, next_block])
                } else {
                    b
                }
            }
            ShapedBlock::Loop(loop_block) => {
                let inner_block = self.build(&loop_block.inner, blocks_ctx);

                let loop_flow = Flow::Loop {
                    types_stack: inner_block.get_types_stack(),
                    loop_id: loop_block.loop_id,
                    body: Box::new(inner_block),
                };
                if let Some(next_block) = &loop_block.next {
                    let next_flow = self.build(next_block, blocks_ctx);
                    Flow::Sequence(vec![loop_flow, next_flow])
                } else {
                    loop_flow
                }
            }
            ShapedBlock::Multiple(multiple_block) => {
                if multiple_block.handled.len() > 2 {
                    panic!("Found ShapedBlock::Multiple with more than 2 branches.");
                }
                let mut handled_blocks = multiple_block
                    .handled
                    .iter()
                    .map(|b| self.build(&b.inner, blocks_ctx))
                    .collect::<Vec<_>>();

                let else_ = handled_blocks.pop().unwrap_or(Flow::Empty);
                let if_ = handled_blocks.pop().unwrap_or(Flow::Empty);
                Flow::IfElse {
                    types_stack: else_.get_types_stack(),
                    then_body: Box::new(if_),
                    else_body: Box::new(else_),
                }
            }
        }
    }
}
