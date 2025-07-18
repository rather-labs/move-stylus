use anyhow::Result;
use walrus::{
    ConstExpr, ElementKind, FunctionId as WasmFunctionId, Module, TableId, TypeId, ValType,
    ir::Value,
};

use crate::compilation_context::ModuleId;

use super::functions::MappedFunction;

/// Identifies a function inside a module
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct FunctionId {
    pub identifier: String,

    pub module_id: ModuleId,
}

pub struct TableEntry {
    pub index: i32,
    pub function_id: FunctionId,
    pub type_id: TypeId,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,

    /// This field is used as a safeguard, it is set to true when `Self::add_to_wasm_table` is
    /// executed. If we find any entry with this field in false, means we never executed said
    /// method, on some entry resulting in some functions not present in the table, if that happens
    /// we are going to be able to call the function present in this entry.
    added_to_wasm_table: bool,
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
        function_id: FunctionId,
        function: &MappedFunction,
    ) -> &TableEntry {
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
            function_id,
            type_id,
            params,
            results,
            added_to_wasm_table: false,
        });

        let table = module.tables.get_mut(self.table_id);
        table.initial = self.entries.len() as u64;

        &self.entries[self.entries.len() - 1]
    }

    pub fn add_to_wasm_table(
        &mut self,
        module: &mut Module,
        index: usize,
        function_id: WasmFunctionId,
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

    pub fn get_by_function_id(&self, function_id: &FunctionId) -> Option<&TableEntry> {
        self.entries.iter().find(|e| &e.function_id == function_id)
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
                "function {:?} was not added to the functions table",
                entry.function_id
            );
        }

        Ok(())
    }
}
