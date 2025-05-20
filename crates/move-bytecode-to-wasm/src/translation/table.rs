use move_binary_format::file_format::FunctionHandleIndex;
use walrus::{ir::Value, ConstExpr, ElementKind, FunctionId, Module, TableId, TypeId, ValType};

use super::functions::MappedFunction;

pub struct TableEntry<'a> {
    pub index: i32,
    pub function: MappedFunction<'a>,
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

pub struct FunctionTable<'a> {
    /// WASM table id
    table_id: TableId,
    entries: Vec<TableEntry<'a>>,
}

impl<'a> FunctionTable<'a> {
    pub fn new(wasm_table_id: TableId) -> Self {
        Self {
            table_id: wasm_table_id,
            entries: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        module: &mut Module,
        function: MappedFunction<'a>,
        params: Vec<ValType>,
        results: Vec<ValType>,
        function_handle_index: FunctionHandleIndex,
    ) {
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

    pub fn get(&self, index: usize) -> Option<&TableEntry<'a>> {
        self.entries.get(index)
    }

    pub fn get_by_function_handle_index(
        &self,
        function_handle_index: &FunctionHandleIndex,
    ) -> Option<&TableEntry<'a>> {
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
}

impl Drop for FunctionTable<'_> {
    fn drop(&mut self) {
        if let Some(entry) = self.entries.iter().find(|e| !e.added_to_wasm_table) {
            panic!(
                "function {} was not added to the functions table",
                entry.function.name
            );
        }
    }
}
