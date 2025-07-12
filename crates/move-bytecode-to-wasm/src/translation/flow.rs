use crate::CompilationContext;
use crate::MappedFunction;
use crate::translation::TypesStack;
use move_abstract_interpreter::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use move_binary_format::file_format::{Bytecode, CodeUnit};
use relooper::{ShapedBlock, reloop};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Flow {
    Simple {
        types_stack: TypesStack,
        label: u16,
        instructions: Vec<Bytecode>, // <- NEW
    },
    Sequence(Vec<Flow>),
    Loop(TypesStack, Box<Flow>),
    IfElse(TypesStack, Box<Flow>, Box<Flow>),
    Empty,
}

impl Flow {
    pub fn get_types_stack(&self) -> TypesStack {
        match self {
            Flow::Simple { types_stack, .. } => types_stack.clone(),
            // im not sure if concat is correct here!
            Flow::Sequence(blocks) => TypesStack(blocks.iter().fold(vec![], |acc, f| {
                [acc, f.get_types_stack().0.clone()].concat()
            })),
            Flow::Loop(types_stack, _) => types_stack.clone(),
            Flow::IfElse(types_stack, _, _) => types_stack.clone(),
            Flow::Empty => TypesStack::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(&self, Self::Empty)
    }
}

pub struct FlowBuilder {
    pub cfg: VMControlFlowGraph,
    pub relooped: ShapedBlock<u16>,
    pub blocks: HashMap<u16, (Vec<Bytecode>, TypesStack)>,
    pub flow: Flow,
}

impl FlowBuilder {
    pub fn new(
        code_unit: &CodeUnit,
        compilation_ctx: &CompilationContext,
        mapped_function: &MappedFunction,
    ) -> Self {
        let cfg = VMControlFlowGraph::new(&code_unit.code, &code_unit.jump_tables);

        let relooped = {
            let nodes: Vec<(u16, Vec<u16>)> = (&cfg as &dyn ControlFlowGraph)
                .blocks()
                .into_iter()
                .map(|b| (b, cfg.successors(b).to_vec()))
                .collect();
            *relooper::reloop(nodes, 0)
        };

        let blocks: HashMap<u16, (Vec<Bytecode>, TypesStack)> = (&cfg as &dyn ControlFlowGraph)
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

        let flow = Self::to_flow(&relooped, &blocks);

        Self {
            cfg,
            relooped,
            blocks,
            flow,
        }
    }

    fn to_flow(
        relooped_cfg: &ShapedBlock<u16>,
        blocks: &HashMap<u16, (Vec<Bytecode>, TypesStack)>,
    ) -> Flow {
        match relooped_cfg {
            ShapedBlock::Simple(simple_block) => {
                let block = blocks.get(&simple_block.label).unwrap();
                let b = Flow::Simple {
                    types_stack: block.1.clone(),
                    label: simple_block.label,
                    instructions: block.0.clone(),
                };

                let inner = simple_block
                    .immediate
                    .as_ref()
                    .map(|i| Self::to_flow(i, blocks))
                    .unwrap_or(Flow::Empty);

                let next = simple_block
                    .next
                    .as_ref()
                    .map(|n| Self::to_flow(n, blocks))
                    .unwrap_or(Flow::Empty);

                if !inner.is_empty() || !next.is_empty() {
                    Flow::Sequence(vec![b, inner, next])
                } else {
                    b
                }
            }
            ShapedBlock::Loop(loop_block) => {
                let inner = Self::to_flow(&loop_block.inner, blocks);
                if let Some(ref next) = loop_block.next {
                    let next = Self::to_flow(next, blocks);
                    Flow::Sequence(vec![inner, next])
                } else {
                    Flow::Loop(inner.get_types_stack(), Box::new(inner))
                }
            }
            ShapedBlock::Multiple(multiple_block) => {
                if multiple_block.handled.len() > 2 {
                    panic!("more than 2 branches not supported");
                }
                let mut processed = multiple_block
                    .handled
                    .iter()
                    .map(|b| Self::to_flow(&b.inner, blocks))
                    .collect::<Vec<_>>();

                let else_ = processed.pop().unwrap_or(Flow::Empty);
                let if_ = processed.pop().unwrap_or(Flow::Empty);
                Flow::IfElse(else_.get_types_stack(), Box::new(if_), Box::new(else_))
            }
        }
    }
}