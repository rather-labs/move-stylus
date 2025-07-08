use move_abstract_interpreter::control_flow_graph::ControlFlowGraph;
use relooper::reloop;
use std::collections::BTreeMap;

use anyhow::Result;
use move_abstract_interpreter::control_flow_graph::VMControlFlowGraph;
use move_binary_format::file_format::{CodeUnit, FunctionHandleIndex};
use walrus::{ConstExpr, ElementKind, FunctionId, Module, TableId, TypeId, ValType, ir::Value};

use super::functions::MappedFunction;

pub struct TableEntry {
    pub index: i32,
    pub function: MappedFunction,
    pub function_handle_index: FunctionHandleIndex,
    pub type_id: TypeId,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,

    /// This field is used as a safeguard, it is set to true when `Self::add_to_wasm_table` is
    /// executed. If we find any entry with this field in false, means we never executed said
    /// method, on some entry resulting in some functions not present in the table, if that happens
    /// we are going to be able to call the function present in this entry.
    added_to_wasm_table: bool,
}

impl TableEntry {
    pub fn get_move_code_unit(&self) -> Option<&CodeUnit> {
        let code_unit = self.function.function_definition.code.as_ref().unwrap();
        let test: &dyn ControlFlowGraph =
            &VMControlFlowGraph::new(&code_unit.code, &code_unit.jump_tables);

        let nodes: Vec<(u16, Vec<u16>)> = test
            .blocks()
            .into_iter()
            .map(|b| (b, test.successors(b).to_vec()))
            .collect();

        println!("========> {nodes:?}");

        let test: BTreeMap<_, _> = VMControlFlowGraph::new(&code_unit.code, &code_unit.jump_tables)
            .blocks()
            .into_iter()
            .enumerate()
            .map(|(block_number, pc_start)| (pc_start, block_number))
            .collect();

        println!("AAAAAAAAAAAAAAAAAAAAAA \n{:#?}", test);

        let relooped = reloop(nodes, 0);
        println!("RELOOPED {relooped:#?}");

        self.function.function_definition.code.as_ref()
    }
}

pub struct FunctionTable {
    /// WASM table id
    table_id: TableId,
    entries: Vec<TableEntry>,
}

impl FunctionTable {
    pub fn new(wasm_table_id: TableId) -> Self {
        Self {
            table_id: wasm_table_id,
            entries: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        module: &mut Module,
        function: MappedFunction,
        function_handle_index: FunctionHandleIndex,
    ) {
        let params: Vec<ValType> = function
            .signature
            .arguments
            .iter()
            .map(ValType::from)
            .collect();

        let results = function.signature.get_return_wasm_types();
        let type_id = module.types.add(&params, &results);
        let index = self.entries.len() as i32;
        self.entries.push(TableEntry {
            index,
            function,
            type_id,
            params,
            results,
            function_handle_index,
            added_to_wasm_table: false,
        });

        let table = module.tables.get_mut(self.table_id);
        table.initial = self.entries.len() as u64;
    }

    pub fn add_to_wasm_table(
        &mut self,
        module: &mut Module,
        index: usize,
        function_id: FunctionId,
    ) -> anyhow::Result<()> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or(anyhow::anyhow!("invalid entry {index}"))?;

        module.elements.add(
            ElementKind::Active {
                table: self.table_id,
                offset: ConstExpr::Value(Value::I32(index as i32)),
            },
            walrus::ElementItems::Functions(vec![function_id]),
        );
        entry.added_to_wasm_table = true;

        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&TableEntry> {
        self.entries.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut TableEntry> {
        self.entries.get_mut(index)
    }

    pub fn get_by_function_handle_index(
        &self,
        function_handle_index: &FunctionHandleIndex,
    ) -> Option<&TableEntry> {
        self.entries
            .iter()
            .find(|e| e.function_handle_index == *function_handle_index)
    }

    pub fn get_table_id(&self) -> TableId {
        self.table_id
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn ensure_all_functions_added(&self) -> Result<()> {
        if let Some(entry) = self.entries.iter().find(|e| !e.added_to_wasm_table) {
            anyhow::bail!(
                "function {} was not added to the functions table",
                entry.function.name
            );
        }

        Ok(())
    }
}
