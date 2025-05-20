use move_binary_format::file_format::FunctionHandleIndex;
use walrus::{Module, TableId, TypeId, ValType};

use super::functions::MappedFunction;

pub struct TableEntry<'a> {
    pub index: i32,
    pub function: MappedFunction<'a>,
    pub function_handle_index: FunctionHandleIndex,
    pub type_id: TypeId,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

pub struct FunctionTable<'a> {
    table_id: TableId,
    entries: Vec<TableEntry<'a>>,

    /// This field is used as a safeguard, it is set to true when `Self::fill_table` is executed.
    /// If the FunctionTable with this field in false, means we never executed said method, so the
    /// functions were never linked to the WASM module. If that happens, we panic!
    table_filled: bool,
}

impl<'a> FunctionTable<'a> {
    pub fn new(wasm_table_id: TableId) -> Self {
        Self {
            table_id: wasm_table_id,
            entries: Vec::new(),
            table_filled: false,
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
        self.entries.push(TableEntry {
            index: self.entries.len() as i32,
            function,
            type_id,
            params,
            results,
            function_handle_index,
        });
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
