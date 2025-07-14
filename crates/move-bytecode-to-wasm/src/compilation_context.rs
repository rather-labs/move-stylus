use std::collections::HashMap;

use crate::translation::{
    functions::MappedFunction,
    intermediate_types::{
        IntermediateType,
        enums::{IEnum, IEnumVariant},
        structs::IStruct,
    },
    table::FunctionTable,
};
use move_binary_format::{
    CompiledModule,
    file_format::{
        Constant, DatatypeHandleIndex, EnumDefinitionIndex, FieldHandleIndex,
        FieldInstantiationIndex, Signature, SignatureIndex, SignatureToken,
        StructDefInstantiationIndex, StructDefinitionIndex, VariantHandleIndex,
    },
    internals::ModuleIndex,
};
use walrus::{FunctionId, MemoryId, RefType};

#[derive(Debug)]
pub enum UserDefinedType {
    Struct(u16),
    Enum(usize),
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ModuleId {
    /// Module we are currently compiling.
    Root,

    /// Dependency module identified by an address.
    Address {
        address: [u8; 32],
        namespace: String,
    },
}

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CompilationContextError {
    #[error("struct with index {0} not found in compilation context")]
    StructNotFound(u16),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithFieldIdxNotFound(FieldHandleIndex),

    #[error("struct with field id {0:?} not found in compilation context")]
    StructWithDefinitionIdxNotFound(StructDefinitionIndex),

    #[error("struct with generic field instance id {0:?} not found in compilation context")]
    GenericStructWithFieldIdxNotFound(FieldInstantiationIndex),

    #[error("generic struct instance with field id {0:?} not found in compilation context")]
    GenericStructWithDefinitionIdxNotFound(StructDefInstantiationIndex),

    #[error("signature with signature index {0:?} not found in compilation context")]
    SignatureNotFound(SignatureIndex),

    #[error("enum with index {0} not found in compilation context")]
    EnumNotFound(u16),

    #[error("enum with enum id {0} not found in compilation context")]
    EnumWithVariantIdxNotFound(u16),
}

#[derive(Debug)]
pub struct VariantData {
    pub enum_index: usize,
    pub index_inside_enum: usize,
}

#[derive(Debug)]
pub struct ModuleData<'a> {
    /// Move's connstant pool
    pub constants: &'a [Constant],

    /// Module's functions arguments.
    pub functions_arguments: &'a [Vec<IntermediateType>],

    /// Module's functions Returns.
    pub functions_returns: &'a [Vec<IntermediateType>],

    /// Module's signatures
    pub module_signatures: &'a [Signature],

    /// Module's structs: contains all the user defined structs
    pub module_structs: &'a [IStruct],

    /// Module's generic structs instances: contains all the user defined generic structs instances
    /// with its corresponding types
    pub module_generic_structs_instances: &'a [(StructDefinitionIndex, Vec<SignatureToken>)],

    /// Maps a field index to its corresponding struct
    pub fields_to_struct_map: &'a HashMap<FieldHandleIndex, StructDefinitionIndex>,

    /// Maps a generic field index to its corresponding struct in module_generic_structs_instances
    pub generic_fields_to_struct_map: &'a HashMap<FieldInstantiationIndex, usize>,

    /// Maps a field instantiation index to its corresponding index inside the struct.
    /// Field instantiation indexes are unique per struct instantiation, so, for example if we have
    /// the following struct:
    /// ```move
    /// struct S<T> {
    ///    x: T,
    /// }
    /// ```
    /// And we instantiate it with `S<u64>`, and `S<bool>`, the we will have a
    /// FieldInstantiationIndex(0) and a FieldInstantiationIndex(1) both for the `x` field, but the
    /// index inside the struct is 0 in both cases.
    ///
    /// We also map the concrete types of the instantiated generic struct where this field
    /// instantiuation belongs to. This is needed because there are situations where we need to
    /// intantiate the struct only with the field instantiation index and no other information.
    pub instantiated_fields_to_generic_fields:
        &'a HashMap<FieldInstantiationIndex, (FieldHandleIndex, Vec<SignatureToken>)>,

    /// Module's enums: contains all the user defined enums
    pub module_enums: &'a [IEnum],

    /// Maps a enum's variant index to its corresponding enum and position inside the enum
    pub variants_to_enum_map: &'a HashMap<VariantHandleIndex, VariantData>,

    /// This Hashmap maps the move's datatype handles to our internal representation of those
    /// types. The datatype handles are used interally by move to look for user defined data
    /// types
    pub datatype_handles_map: &'a HashMap<DatatypeHandleIndex, UserDefinedType>,
}

/// Compilation context
///
/// Functions are processed in order. To access function information (i.e: arguments or return
/// arguments we must know the index of it)
pub struct CompilationContext<'a> {
    /// Data of the module we are currently compiling
    pub root_module_data: ModuleData<'a>,

    pub deps_data: HashMap<ModuleId, ModuleData<'a>>,

    /// WASM memory id
    pub memory_id: MemoryId,

    /// Allocator function id
    pub allocator: FunctionId,
}

impl CompilationContext<'_> {
    ///  Creates a ModuleData for a module dependency. Each dependency is identified by an address
    ///  and, under that address there can be defined several namespaces.
    /*
    pub fn process_dependency_module<'a>(
        module_handle: &ModuleHandle,
        root_module: &'a CompiledModule,
    ) -> (ModuleId, ModuleData<'a>) {
        let ModuleHandle { address, name } = module_handle;

        let module_id = ModuleId::Address {
            namespace: root_module.identifier_at(*name).to_string(),
            address: root_module.address_identifier_at(*address).into_bytes(),
        };
    }
    */

    pub fn process_datatype_handles(
        module: &CompiledModule,
    ) -> HashMap<DatatypeHandleIndex, UserDefinedType> {
        let mut datatype_handles_map = HashMap::new();

        for (index, datatype_handle) in module.datatype_handles().iter().enumerate() {
            let idx = DatatypeHandleIndex::new(index as u16);

            // Assert the index we constructed is ok
            assert_eq!(datatype_handle, module.datatype_handle_at(idx));

            if let Some(position) = module
                .struct_defs()
                .iter()
                .position(|s| s.struct_handle == idx)
            {
                datatype_handles_map.insert(idx, UserDefinedType::Struct(position as u16));
            } else if let Some(position) =
                module.enum_defs().iter().position(|e| e.enum_handle == idx)
            {
                datatype_handles_map.insert(idx, UserDefinedType::Enum(position));
            } else {
                panic!("datatype handle index {index} not found");
            };
        }

        datatype_handles_map
    }

    pub fn process_concrete_structs(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> (
        Vec<IStruct>,
        HashMap<FieldHandleIndex, StructDefinitionIndex>,
    ) {
        // Module's structs
        let mut module_structs: Vec<IStruct> = vec![];
        let mut fields_to_struct_map = HashMap::new();
        for (index, struct_def) in module.struct_defs().iter().enumerate() {
            let struct_index = StructDefinitionIndex::new(index as u16);
            let mut fields_map = HashMap::new();
            let mut all_fields = Vec::new();
            if let Some(fields) = struct_def.fields() {
                for (field_index, field) in fields.iter().enumerate() {
                    let intermediate_type = IntermediateType::try_from_signature_token(
                        &field.signature.0,
                        datatype_handles_map,
                    )
                    .unwrap();

                    let field_index = module
                        .field_handles()
                        .iter()
                        .position(|f| f.field == field_index as u16 && f.owner == struct_index)
                        .map(|i| FieldHandleIndex::new(i as u16));

                    // If field_index is None means the field is never referenced in the code
                    if let Some(field_index) = field_index {
                        let res = fields_map.insert(field_index, intermediate_type.clone());
                        assert!(
                            res.is_none(),
                            "there was an error creating a field in struct {struct_index}, field with index {field_index} already exist"
                        );
                        let res = fields_to_struct_map.insert(field_index, struct_index);
                        assert!(
                            res.is_none(),
                            "there was an error mapping field {field_index} to struct {struct_index}, already mapped"
                        );
                        all_fields.push((Some(field_index), intermediate_type));
                    } else {
                        all_fields.push((None, intermediate_type));
                    }
                }
            }

            module_structs.push(IStruct::new(struct_index, all_fields, fields_map));
        }

        (module_structs, fields_to_struct_map)
    }

    pub fn process_generic_structs(
        module: &CompiledModule,
    ) -> (
        Vec<(StructDefinitionIndex, Vec<SignatureToken>)>,
        HashMap<FieldInstantiationIndex, usize>,
    ) {
        let mut module_generic_structs_instances = vec![];
        let mut generic_fields_to_struct_map = HashMap::new();

        for (index, struct_instance) in module.struct_instantiations().iter().enumerate() {
            // Map the struct instantiation to the generic struct definition and the instantiation
            // types. The index in the array will match the PackGeneric(index) instruction
            let struct_instantiation_types =
                &module.signature_at(struct_instance.type_parameters).0;

            module_generic_structs_instances
                .push((struct_instance.def, struct_instantiation_types.clone()));

            // Process the mapping of generic fields to structs instantiations
            let generic_struct_definition = &module.struct_defs()[struct_instance.def.0 as usize];

            let struct_index = StructDefinitionIndex::new(struct_instance.def.0);
            let generic_struct_index = StructDefInstantiationIndex::new(index as u16);

            if let Some(fields) = generic_struct_definition.fields() {
                for (field_index, _) in fields.iter().enumerate() {
                    let generic_field_index = module
                        .field_instantiations()
                        .iter()
                        .position(|f| {
                            let field_handle = &module.field_handle_at(f.handle);
                            let struct_def_instantiation =
                                &module.struct_instantiation_at(generic_struct_index);

                            // Filter which generic field we are processing inside the struct
                            field_handle.field == field_index as u16
                                // Link it with the generic struct definition
                                && field_handle.owner == struct_index
                                // Link it with the struct instantiation using the signature
                                && struct_def_instantiation.type_parameters == f.type_parameters
                        })
                        .map(|i| FieldInstantiationIndex::new(i as u16));

                    // If field_index is None means the field is never referenced in the code
                    if let Some(generic_field_index) = generic_field_index {
                        let res = generic_fields_to_struct_map.insert(generic_field_index, index);
                        assert!(
                            res.is_none(),
                            "there was an error mapping field {generic_field_index} to struct {struct_index}, already mapped"
                        );
                    }
                }
            }
        }

        (
            module_generic_structs_instances,
            generic_fields_to_struct_map,
        )
    }

    pub fn process_generic_field_instances(
        module: &CompiledModule,
    ) -> HashMap<FieldInstantiationIndex, (FieldHandleIndex, Vec<SignatureToken>)> {
        // Map instantiated struct fields to indexes of generic fields
        let mut instantiated_fields_to_generic_fields = HashMap::new();
        for (index, field_instance) in module.field_instantiations().iter().enumerate() {
            instantiated_fields_to_generic_fields.insert(
                FieldInstantiationIndex::new(index as u16),
                (
                    field_instance.handle,
                    module
                        .signature_at(field_instance.type_parameters)
                        .0
                        .clone(),
                ),
            );
        }
        instantiated_fields_to_generic_fields
    }

    pub fn process_concrete_enums(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> (Vec<IEnum>, HashMap<VariantHandleIndex, VariantData>) {
        // Module's enums
        let mut module_enums = vec![];
        let mut variants_to_enum_map = HashMap::new();
        for (index, enum_def) in module.enum_defs().iter().enumerate() {
            let enum_index = EnumDefinitionIndex::new(index as u16);
            let mut variants = Vec::new();

            // Process variants
            for (variant_index, variant) in enum_def.variants.iter().enumerate() {
                let fields = variant
                    .fields
                    .iter()
                    .map(|f| {
                        IntermediateType::try_from_signature_token(
                            &f.signature.0,
                            datatype_handles_map,
                        )
                    })
                    .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap();

                variants.push(IEnumVariant::new(
                    variant_index as u16,
                    index as u16,
                    fields,
                ));

                // Process handles
                let variant_handle_index = module
                    .variant_handles()
                    .iter()
                    .position(|v| v.variant == variant_index as u16 && v.enum_def == enum_index)
                    .map(|i| VariantHandleIndex(i as u16));

                // If variant_handle_index is None means the field is never referenced in the code
                if let Some(variant_handle_index) = variant_handle_index {
                    let res = variants_to_enum_map.insert(
                        variant_handle_index,
                        VariantData {
                            enum_index: index,
                            index_inside_enum: variant_index,
                        },
                    );
                    assert!(
                        res.is_none(),
                        "there was an error creating a variant in struct {variant_index}, variant with index {variant_index} already exist"
                    );
                }
            }

            module_enums.push(IEnum::new(index as u16, variants).unwrap());
        }

        (module_enums, variants_to_enum_map)
    }
    pub fn process_function_definitions(
        move_module: &CompiledModule,
        wasm_module: &mut walrus::Module,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> (
        FunctionTable,
        Vec<Vec<IntermediateType>>,
        Vec<Vec<IntermediateType>>,
    ) {
        // Return types of functions in intermediate types. Used to fill the stack type
        let mut functions_returns = Vec::new();
        let mut functions_arguments = Vec::new();

        // Function table
        let function_table_id = wasm_module
            .tables
            .add_local(false, 0, None, RefType::Funcref);
        let mut function_table = FunctionTable::new(function_table_id);

        for (function_def, function_handle) in move_module
            .function_defs()
            .iter()
            .zip(move_module.function_handles.iter())
        {
            let move_function_arguments = &move_module.signature_at(function_handle.parameters);

            functions_arguments.push(
                move_function_arguments
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                    .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let move_function_return = &move_module.signature_at(function_handle.return_);

            functions_returns.push(
                move_function_return
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                    .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let code_locals = &move_module.signature_at(function_def.code.as_ref().unwrap().locals);

            let function_name = move_module.identifier_at(function_handle.name).to_string();

            let function_handle_index = function_def.function;
            let mapped_function = MappedFunction::new(
                function_name,
                move_function_arguments,
                move_function_return,
                code_locals,
                function_def.clone(), // TODO: check clone
                datatype_handles_map,
                wasm_module,
            );

            function_table.add(wasm_module, mapped_function, function_handle_index);
        }

        (function_table, functions_arguments, functions_returns)
    }

    pub fn get_struct_by_index(&self, index: u16) -> Result<&IStruct, CompilationContextError> {
        self.root_module_data
            .module_structs
            .iter()
            .find(|s| s.index() == index)
            .ok_or(CompilationContextError::StructNotFound(index))
    }

    pub fn get_struct_by_field_handle_idx(
        &self,
        field_index: &FieldHandleIndex,
    ) -> Result<&IStruct, CompilationContextError> {
        let struct_id = self
            .root_module_data
            .fields_to_struct_map
            .get(field_index)
            .ok_or(CompilationContextError::StructWithFieldIdxNotFound(
                *field_index,
            ))?;

        self.root_module_data
            .module_structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_id)
            .ok_or(CompilationContextError::StructWithFieldIdxNotFound(
                *field_index,
            ))
    }

    pub fn get_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefinitionIndex,
    ) -> Result<&IStruct, CompilationContextError> {
        self.root_module_data
            .module_structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_index)
            .ok_or(CompilationContextError::StructWithDefinitionIdxNotFound(
                *struct_index,
            ))
    }

    pub fn get_generic_struct_by_field_handle_idx(
        &self,
        field_index: &FieldInstantiationIndex,
    ) -> Result<IStruct, CompilationContextError> {
        let struct_id = self
            .root_module_data
            .generic_fields_to_struct_map
            .get(field_index)
            .ok_or(CompilationContextError::GenericStructWithFieldIdxNotFound(
                *field_index,
            ))?;

        let struct_instance = &self.root_module_data.module_generic_structs_instances[*struct_id];
        let generic_struct = &self.root_module_data.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| {
                IntermediateType::try_from_signature_token(
                    t,
                    self.root_module_data.datatype_handles_map,
                )
            })
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<IStruct, CompilationContextError> {
        let struct_instance =
            &self.root_module_data.module_generic_structs_instances[struct_index.0 as usize];
        let generic_struct = &self.root_module_data.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| {
                IntermediateType::try_from_signature_token(
                    t,
                    self.root_module_data.datatype_handles_map,
                )
            })
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_types_instances(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<Vec<IntermediateType>, CompilationContextError> {
        let struct_instance =
            &self.root_module_data.module_generic_structs_instances[struct_index.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| {
                IntermediateType::try_from_signature_token(
                    t,
                    self.root_module_data.datatype_handles_map,
                )
            })
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(types)
    }

    pub fn get_generic_struct_idx_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> u16 {
        let struct_instance =
            &self.root_module_data.module_generic_structs_instances[struct_index.0 as usize];
        struct_instance.0.0
    }

    pub fn get_signatures_by_index(
        &self,
        index: SignatureIndex,
    ) -> Result<&Vec<SignatureToken>, CompilationContextError> {
        self.root_module_data
            .module_signatures
            .get(index.into_index())
            .map(|s| &s.0)
            .ok_or(CompilationContextError::SignatureNotFound(index))
    }

    pub fn get_enum_by_variant_handle_idx(
        &self,
        idx: &VariantHandleIndex,
    ) -> Result<&IEnum, CompilationContextError> {
        let VariantData { enum_index, .. } = self
            .root_module_data
            .variants_to_enum_map
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        self.root_module_data
            .module_enums
            .get(*enum_index)
            .ok_or(CompilationContextError::EnumNotFound(*enum_index as u16))
    }

    pub fn get_variant_position_by_variant_handle_idx(
        &self,
        idx: &VariantHandleIndex,
    ) -> Result<u16, CompilationContextError> {
        let VariantData {
            index_inside_enum, ..
        } = self
            .root_module_data
            .variants_to_enum_map
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        Ok(*index_inside_enum as u16)
    }

    pub fn get_enum_by_index(&self, index: u16) -> Result<&IEnum, CompilationContextError> {
        self.root_module_data
            .module_enums
            .get(index as usize)
            .ok_or(CompilationContextError::EnumNotFound(index))
    }
}
