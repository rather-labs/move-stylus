use std::fmt::Display;

use anyhow::Result;
use walrus::{
    ConstExpr, ElementKind, FunctionId as WasmFunctionId, Module, TableId, TypeId, ValType,
    ir::Value,
};

use crate::compilation_context::ModuleId;

use super::{functions::MappedFunction, intermediate_types::IntermediateType};

/// Identifies a function inside a module
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct FunctionId {
    pub identifier: String,

    pub module_id: ModuleId,

    pub type_instantiations: Option<Vec<IntermediateType>>,
}

impl FunctionId {
    /// Returns the generic function ID corresponding to a function ID with type instantiations.
    pub fn get_generic_fn_id(&self) -> Self {
        // TODO: clone...
        let mut id = self.clone();
        id.type_instantiations = None;
        id
    }
}

impl Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.module_id, self.identifier)
    }
}

#[derive(Debug)]
pub struct TableEntry {
    pub index: i32,
    pub function_id: FunctionId,
    pub wasm_function_id: Option<WasmFunctionId>,
    pub type_id: TypeId,
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
            wasm_function_id: None,
            type_id,
        });

        let table = module.tables.get_mut(self.table_id);
        table.initial = self.entries.len() as u64;

        &self.entries[self.entries.len() - 1]
    }

    pub fn add_to_wasm_table(
        &mut self,
        module: &mut Module,
        function_id: &FunctionId,
        wasm_function_id: WasmFunctionId,
    ) -> anyhow::Result<()> {
        let entry = self
            .get_by_function_id(function_id)
            .ok_or(anyhow::anyhow!("invalid entry {function_id:?}"))?;

        module.elements.add(
            ElementKind::Active {
                table: self.table_id,
                offset: ConstExpr::Value(Value::I32(entry.index as i32)),
            },
            walrus::ElementItems::Functions(vec![wasm_function_id]),
        );

        let entry = self
            .get_mut_by_function_id(function_id)
            .ok_or(anyhow::anyhow!("invalid entry {function_id:?}"))?;

        entry.wasm_function_id = Some(wasm_function_id);

        Ok(())
    }

    pub fn get_by_function_id(&self, function_id: &FunctionId) -> Option<&TableEntry> {
        self.entries.iter().find(|e| &e.function_id == function_id)
    }

    pub fn get_mut_by_function_id(&mut self, function_id: &FunctionId) -> Option<&mut TableEntry> {
        self.entries
            .iter_mut()
            .find(|e| &e.function_id == function_id)
    }

    pub fn get_table_id(&self) -> TableId {
        self.table_id
    }

    pub fn ensure_all_functions_added(&self) -> Result<()> {
        if let Some(entry) = self.entries.iter().find(|e| e.wasm_function_id.is_none()) {
            anyhow::bail!(
                "function {} was not added to the functions table",
                entry.function_id
            );
        }

        Ok(())
    }
}
