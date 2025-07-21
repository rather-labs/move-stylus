use crate::{
    GlobalFunctionTable,
    translation::{
        functions::MappedFunction,
        intermediate_types::{
            IntermediateType,
            enums::{IEnum, IEnumVariant},
            structs::IStruct,
        },
        table::FunctionId,
    },
};
use move_binary_format::{
    CompiledModule,
    file_format::{
        Constant, DatatypeHandleIndex, EnumDefinitionIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinitionIndex, Signature, SignatureIndex,
        SignatureToken, StructDefInstantiationIndex, StructDefinitionIndex, VariantHandleIndex,
    },
    internals::ModuleIndex,
};
use std::{collections::HashMap, fmt::Display};

use super::{CompilationContextError, Result};

#[derive(Debug)]
pub enum UserDefinedType {
    /// Struct defined in this module
    Struct(u16),

    /// Enum defined in this module
    Enum(usize),

    /// Data type defined outside this module
    ExternalData {
        module: ModuleId,
        identifier: String,
    },
}

#[derive(Debug)]
pub struct VariantData {
    pub enum_index: usize,
    pub index_inside_enum: usize,
}

#[derive(Debug, Default)]
pub struct ModuleData {
    /// Move's connstant pool
    pub constants: Vec<Constant>,

    /// Module's functions arguments.
    pub functions_arguments: Vec<Vec<IntermediateType>>,

    /// Module's functions Returns.
    pub functions_returns: Vec<Vec<IntermediateType>>,

    /// Module's signatures
    pub module_signatures: Vec<Signature>,

    /// Module's structs: contains all the user defined structs
    pub module_structs: Vec<IStruct>,

    /// Module's generic structs instances: contains all the user defined generic structs instances
    /// with its corresponding types
    pub module_generic_structs_instances: Vec<(StructDefinitionIndex, Vec<SignatureToken>)>,

    /// Maps a field index to its corresponding struct
    pub fields_to_struct_map: HashMap<FieldHandleIndex, StructDefinitionIndex>,

    /// Maps a generic field index to its corresponding struct in module_generic_structs_instances
    pub generic_fields_to_struct_map: HashMap<FieldInstantiationIndex, usize>,

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
        HashMap<FieldInstantiationIndex, (FieldHandleIndex, Vec<SignatureToken>)>,

    /// Module's enums: contains all the user defined enums
    pub module_enums: Vec<IEnum>,

    /// Maps a enum's variant index to its corresponding enum and position inside the enum
    pub variants_to_enum_map: HashMap<VariantHandleIndex, VariantData>,

    /// This Hashmap maps the move's datatype handles to our internal representation of those
    /// types. The datatype handles are used interally by move to look for user defined data
    /// types
    pub datatype_handles_map: HashMap<DatatypeHandleIndex, UserDefinedType>,

    /// Functions called inside this module. The functions on this list can be defined inside the
    /// current module or in an immediate dependency
    pub function_calls: Vec<FunctionId>,

    /// Function information about this module's defined functions
    pub function_information: Vec<MappedFunction>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Address([u8; 32]);

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(last_nonzero) = self.0.iter().rposition(|&b| b != 0) {
            for byte in &self.0[last_nonzero..] {
                write!(f, "0x{:02x}", byte)?;
            }
        } else {
            write!(f, "0x0")?;
        }

        Ok(())
    }
}

impl From<[u8; 32]> for Address {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ModuleId {
    pub address: Address,
    pub module_name: String,
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.address, self.module_name)
    }
}

impl ModuleData {
    pub fn build_module_data<'move_package>(
        module_id: ModuleId,
        move_module: &'move_package CompiledModule,
        function_definitions: &mut GlobalFunctionTable<'move_package>,
    ) -> Self {
        let datatype_handles_map = Self::process_datatype_handles(move_module);

        let (module_generic_structs_instances, generic_fields_to_struct_map) =
            Self::process_generic_structs(move_module);

        let instantiated_fields_to_generic_fields =
            Self::process_generic_field_instances(move_module);

        // Module's structs
        let (module_structs, fields_to_struct_map) =
            Self::process_concrete_structs(move_module, &datatype_handles_map);

        // Module's enums
        let (module_enums, variants_to_enum_map) =
            Self::process_concrete_enums(move_module, &datatype_handles_map);

        let (functions_arguments, functions_returns, function_calls, function_information) =
            Self::process_function_definitions(
                module_id,
                move_module,
                &datatype_handles_map,
                function_definitions,
            );

        ModuleData {
            constants: move_module.constant_pool.clone(), // TODO: Clone
            functions_arguments,
            functions_returns,
            module_signatures: move_module.signatures.clone(),
            module_structs,
            module_generic_structs_instances,
            datatype_handles_map,
            fields_to_struct_map,
            generic_fields_to_struct_map,
            module_enums,
            variants_to_enum_map,
            instantiated_fields_to_generic_fields,
            function_calls,
            function_information,
        }
    }

    fn process_datatype_handles(
        module: &CompiledModule,
    ) -> HashMap<DatatypeHandleIndex, UserDefinedType> {
        let mut datatype_handles_map = HashMap::new();

        for (index, datatype_handle) in module.datatype_handles().iter().enumerate() {
            let idx = DatatypeHandleIndex::new(index as u16);

            // Assert the index we constructed is ok
            assert_eq!(datatype_handle, module.datatype_handle_at(idx));

            // Check if the datatype is constructed in this module.
            if datatype_handle.module == module.self_handle_idx() {
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
            } else {
                let datatype_module = module.module_handle_at(datatype_handle.module);
                let module_id = ModuleId {
                    address: module
                        .address_identifier_at(datatype_module.address)
                        .into_bytes()
                        .into(),
                    module_name: module.identifier_at(datatype_module.name).to_string(),
                };

                datatype_handles_map.insert(
                    idx,
                    UserDefinedType::ExternalData {
                        module: module_id,
                        identifier: module.identifier_at(datatype_handle.name).to_string(),
                    },
                );
            }
        }

        datatype_handles_map
    }

    fn process_concrete_structs(
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

            let identifier = module
                .identifier_at(module.datatype_handle_at(struct_def.struct_handle).name)
                .to_string();

            module_structs.push(IStruct::new(
                struct_index,
                identifier,
                all_fields,
                fields_map,
            ));
        }

        (module_structs, fields_to_struct_map)
    }

    #[allow(clippy::type_complexity)]
    fn process_generic_structs(
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

    fn process_generic_field_instances(
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
                    .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
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

    #[allow(clippy::type_complexity)]
    fn process_function_definitions<'move_package>(
        module_id: ModuleId,
        move_module: &'move_package CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        function_definitions: &mut GlobalFunctionTable<'move_package>,
    ) -> (
        Vec<Vec<IntermediateType>>,
        Vec<Vec<IntermediateType>>,
        Vec<FunctionId>,
        Vec<MappedFunction>,
    ) {
        // Return types of functions in intermediate types. Used to fill the stack type
        let mut functions_returns = Vec::new();
        let mut functions_arguments = Vec::new();
        let mut function_calls = Vec::new();
        let mut function_information = Vec::new();

        for (index, function) in move_module.function_handles().iter().enumerate() {
            let move_function_arguments = &move_module.signature_at(function.parameters);

            functions_arguments.push(
                move_function_arguments
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                    .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let move_function_return = &move_module.signature_at(function.return_);

            functions_returns.push(
                move_function_return
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                    .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
                    .unwrap(),
            );

            let function_name = move_module.identifier_at(function.name).as_str();

            let function_module = move_module.module_handle_at(function.module);
            let function_module_name = move_module.identifier_at(function_module.name).as_str();
            let function_module_address: Address = move_module
                .address_identifier_at(function_module.address)
                .into_bytes()
                .into();

            // TODO: clones and to_string()....
            let function_id = FunctionId {
                identifier: function_name.to_string(),
                module_id: ModuleId {
                    address: function_module_address,
                    module_name: function_module_name.to_string(),
                },
            };

            // If the functions is defined in this module, we can obtain its definition and process
            // it.
            // If the function is not defined here, it will be processed when processing the
            // dependency
            if function_module_name == module_id.module_name
                && function_module_address == module_id.address
            {
                let function_def =
                    move_module.function_def_at(FunctionDefinitionIndex::new(index as u16));

                assert!(
                    function_def.acquires_global_resources.is_empty(),
                    "Acquiring global resources is not supported yet"
                );
                // Code can be empty (for example in native functions)
                let code_locals = if let Some(code) = function_def.code.as_ref() {
                    &move_module.signature_at(code.locals).0
                } else {
                    &vec![]
                };

                function_information.push(MappedFunction::new(
                    function_id.clone(),
                    move_function_arguments,
                    move_function_return,
                    code_locals,
                    function_def,
                    datatype_handles_map,
                ));

                function_definitions.insert(function_id.clone(), function_def);
            }

            function_calls.push(function_id);
        }

        (
            functions_arguments,
            functions_returns,
            function_calls,
            function_information,
        )
    }

    // =======
    // Structs
    // =======

    pub fn get_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefinitionIndex,
    ) -> Result<&IStruct> {
        self.module_structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_index)
            .ok_or(CompilationContextError::StructWithDefinitionIdxNotFound(
                *struct_index,
            ))
    }

    pub fn get_generic_struct_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<IStruct> {
        let struct_instance = &self.module_generic_structs_instances[struct_index.0 as usize];
        let generic_struct = &self.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| IntermediateType::try_from_signature_token(t, &self.datatype_handles_map))
            .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_struct_by_index(&self, index: u16) -> Result<&IStruct> {
        self.module_structs
            .iter()
            .find(|s| s.index() == index)
            .ok_or(CompilationContextError::StructNotFound(index))
    }

    pub fn get_struct_by_field_handle_idx(
        &self,
        field_index: &FieldHandleIndex,
    ) -> Result<&IStruct> {
        let struct_id = self.fields_to_struct_map.get(field_index).ok_or(
            CompilationContextError::StructWithFieldIdxNotFound(*field_index),
        )?;

        self.module_structs
            .iter()
            .find(|s| &s.struct_definition_index == struct_id)
            .ok_or(CompilationContextError::StructWithFieldIdxNotFound(
                *field_index,
            ))
    }

    // ===============
    // Generic Structs
    // ===============

    pub fn get_generic_struct_by_field_handle_idx(
        &self,
        field_index: &FieldInstantiationIndex,
    ) -> Result<IStruct> {
        let struct_id = self.generic_fields_to_struct_map.get(field_index).ok_or(
            CompilationContextError::GenericStructWithFieldIdxNotFound(*field_index),
        )?;

        let struct_instance = &self.module_generic_structs_instances[*struct_id];
        let generic_struct = &self.module_structs[struct_instance.0.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| IntermediateType::try_from_signature_token(t, &self.datatype_handles_map))
            .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(generic_struct.instantiate(&types))
    }

    pub fn get_generic_struct_types_instances(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> Result<Vec<IntermediateType>> {
        let struct_instance = &self.module_generic_structs_instances[struct_index.0 as usize];

        let types = struct_instance
            .1
            .iter()
            .map(|t| IntermediateType::try_from_signature_token(t, &self.datatype_handles_map))
            .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
            .unwrap();

        Ok(types)
    }

    pub fn get_generic_struct_idx_by_struct_definition_idx(
        &self,
        struct_index: &StructDefInstantiationIndex,
    ) -> u16 {
        let struct_instance = &self.module_generic_structs_instances[struct_index.0 as usize];
        struct_instance.0.0
    }

    pub fn get_enum_by_variant_handle_idx(&self, idx: &VariantHandleIndex) -> Result<&IEnum> {
        let VariantData { enum_index, .. } = self
            .variants_to_enum_map
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        self.module_enums
            .get(*enum_index)
            .ok_or(CompilationContextError::EnumNotFound(*enum_index as u16))
    }

    // =====
    // Enums
    // =====

    pub fn get_variant_position_by_variant_handle_idx(
        &self,
        idx: &VariantHandleIndex,
    ) -> Result<u16> {
        let VariantData {
            index_inside_enum, ..
        } = self
            .variants_to_enum_map
            .get(idx)
            .ok_or(CompilationContextError::EnumWithVariantIdxNotFound(idx.0))?;

        Ok(*index_inside_enum as u16)
    }

    pub fn get_enum_by_index(&self, index: u16) -> Result<&IEnum> {
        self.module_enums
            .get(index as usize)
            .ok_or(CompilationContextError::EnumNotFound(index))
    }

    // ====
    // General
    // ==

    pub fn get_signatures_by_index(&self, index: SignatureIndex) -> Result<&Vec<SignatureToken>> {
        self.module_signatures
            .get(index.into_index())
            .map(|s| &s.0)
            .ok_or(CompilationContextError::SignatureNotFound(index))
    }
}
