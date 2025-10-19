use move_abstract_interpreter::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use move_binary_format::file_format::{Bytecode, CodeUnit};
use relooper::{BranchMode, ShapedBlock};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum Flow {
    Simple {
        label: u16,
        instructions: Vec<Bytecode>,
        immediate: Box<Flow>,
        next: Box<Flow>,
        branches: HashMap<u16, BranchMode>,
    },
    Loop {
        loop_id: u16,
        inner: Box<Flow>,
        next: Box<Flow>,
    },
    IfElse {
        then_body: Box<Flow>,
        else_body: Box<Flow>,
    },
    Switch {
        cases: Vec<Flow>,
    },
    Empty,
}

impl Flow {
    pub fn new(code_unit: &CodeUnit) -> Flow {
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

        // Context for each block within the control flow graph
        let blocks_ctx: HashMap<u16, Vec<Bytecode>> = (&cfg as &dyn ControlFlowGraph)
            .blocks()
            .into_iter()
            .map(|b| {
                let start = cfg.block_start(b);
                let end = cfg.block_end(b) + 1;
                let code = &code_unit.code[start as usize..end as usize];

                (start, code.to_vec())
            })
            .collect();

        Self::build(&relooped, &blocks_ctx)
    }

    fn build(shaped_block: &ShapedBlock<u16>, blocks_ctx: &HashMap<u16, Vec<Bytecode>>) -> Flow {
        match shaped_block {
            ShapedBlock::Simple(simple_block) => {
                let simple_block_ctx = blocks_ctx.get(&simple_block.label).unwrap();

                // `Immediate` blocks are dominated by the current block
                let immediate_flow = simple_block
                    .immediate
                    .as_ref()
                    .map(|b| Self::build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                // `Next` is the structured continuation after the current block, but not necessarily dominated by it.
                let next_flow = simple_block
                    .next
                    .as_ref()
                    .map(|b| Self::build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                // `Branches` represent the control flow edges from the current block to other blocks.
                let branches: HashMap<u16, BranchMode> = simple_block
                    .branches
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .collect();

                assert_eq!(
                    branches.len(),
                    branches.keys().collect::<HashSet<_>>().len()
                );

                Flow::Simple {
                    label: simple_block.label,
                    instructions: simple_block_ctx.clone(),
                    immediate: Box::new(immediate_flow),
                    next: Box::new(next_flow),
                    branches,
                }
            }
            ShapedBlock::Loop(loop_block) => {
                let inner_flow = Self::build(&loop_block.inner, blocks_ctx);

                let next_flow = loop_block
                    .next
                    .as_ref()
                    .map(|b| Self::build(b, blocks_ctx))
                    .unwrap_or(Flow::Empty);

                Flow::Loop {
                    loop_id: loop_block.loop_id,
                    inner: Box::new(inner_flow),
                    next: Box::new(next_flow),
                }
            }
            ShapedBlock::Multiple(multiple_block) => {
                // The relooper algorithm generates multiple blocks when a conditional jump is present.
                // Based on observations, these multiple blocks typically have 1 or 2 handled blocks (branches).
                // For instance, a Move function with a multi-branch match statement is transformed into nested multiple blocks.
                // Alternatively, multiple blocks with a single branch can occur when there is no else condition --> while (true) { if (condition) { break } }
                // Enums add complexity, introducing multiple blocks with more than two branches, typically due to match statements.
                match multiple_block.handled.len() {
                    // If there is a single branch, then instead of creating an if/else flow with an empty arm, we just build the flow from the only handled block.
                    1 => Self::build(&multiple_block.handled[0].inner, blocks_ctx),
                    // If there are two branches, we create an if/else flow with the two handled blocks.
                    2 => {
                        let then_arm = Self::build(&multiple_block.handled[0].inner, blocks_ctx);
                        let else_arm = Self::build(&multiple_block.handled[1].inner, blocks_ctx);

                        Flow::IfElse {
                            then_body: Box::new(then_arm),
                            else_body: Box::new(else_arm),
                        }
                    }
                    _ => {
                        // Build all arms
                        let cases: Vec<Flow> = multiple_block
                            .handled
                            .iter()
                            .map(|b| Self::build(&b.inner, blocks_ctx))
                            .collect();

                        // Assumption: all cases are Simple.
                        // If this is not the case, panic.
                        // This is useful because we can get the label of the case and use it later on to translate the case.
                        assert!(
                            cases.iter().all(|c| matches!(c, Flow::Simple { .. })),
                            "All cases must be Simple in a Switch flow"
                        );

                        Flow::Switch { cases }
                    }
                }
            }
        }
    }

    pub fn get_label(&self) -> u16 {
        match self {
            Flow::Simple { label, .. } => *label,
            _ => panic!("Only Simple flow has label"),
        }
    }

    /// Helper function to check if the flow contains a Ret instruction inside it.
    /// This is used to determine the result type of the block.
    pub fn dominates_return(&self) -> bool {
        match self {
            Flow::Simple {
                instructions,
                immediate,
                next,
                ..
            } => {
                instructions
                    .last()
                    .is_some_and(|b| matches!(b, Bytecode::Ret))
                    || immediate.dominates_return()
                    || next.dominates_return()
            }
            Flow::Loop { inner, next, .. } => inner.dominates_return() || next.dominates_return(),
            Flow::IfElse {
                then_body,
                else_body,
                ..
            } => then_body.dominates_return() || else_body.dominates_return(),
            Flow::Switch { cases } => cases.iter().any(|c| c.dominates_return()),
            Flow::Empty => false,
        }
    }
}
