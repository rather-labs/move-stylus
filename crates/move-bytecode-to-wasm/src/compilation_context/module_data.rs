mod enum_data;
mod function_data;
mod struct_data;

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
use enum_data::{EnumData, VariantData};
use function_data::FunctionData;
use move_binary_format::{
    CompiledModule,
    file_format::{
        Constant, DatatypeHandleIndex, EnumDefinitionIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinitionIndex, SignatureIndex,
        StructDefInstantiationIndex, StructDefinitionIndex, VariantHandleIndex,
    },
    internals::ModuleIndex,
};
use std::{collections::HashMap, fmt::Display};
use struct_data::StructData;

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

#[derive(Debug, Default)]
pub struct ModuleData {
    /// Move's connstant pool
    pub constants: Vec<Constant>,

    /// Module's functions information
    pub functions: FunctionData,

    /// Module's structs information
    pub structs: StructData,

    /// Module's enum information
    pub enums: EnumData,

    /// Module's signatures
    pub signatures: Vec<Vec<IntermediateType>>,

    /// This Hashmap maps the move's datatype handles to our internal representation of those
    /// types. The datatype handles are used interally by move to look for user defined data
    /// types
    pub datatype_handles_map: HashMap<DatatypeHandleIndex, UserDefinedType>,
}

impl ModuleData {
    pub fn build_module_data<'move_package>(
        module_id: ModuleId,
        move_module: &'move_package CompiledModule,
        function_definitions: &mut GlobalFunctionTable<'move_package>,
    ) -> Self {
        let datatype_handles_map = Self::process_datatype_handles(move_module);

        // Module's structs
        let (module_structs, fields_to_struct_map) =
            Self::process_concrete_structs(move_module, &datatype_handles_map);

        let (module_generic_structs_instances, generic_fields_to_struct_map) =
            Self::process_generic_structs(move_module, &datatype_handles_map);

        let instantiated_fields_to_generic_fields =
            Self::process_generic_field_instances(move_module, &datatype_handles_map);

        let structs = StructData {
            structs: module_structs,
            generic_structs_instances: module_generic_structs_instances,
            fields_to_struct: fields_to_struct_map,
            generic_fields_to_struct: generic_fields_to_struct_map,
            instantiated_fields_to_generic_fields,
        };

        // Module's enums
        let (module_enums, variants_to_enum_map) =
            Self::process_concrete_enums(move_module, &datatype_handles_map);

        let enums = EnumData {
            enums: module_enums,
            variants_to_enum: variants_to_enum_map,
        };

        let functions = Self::process_function_definitions(
            module_id,
            move_module,
            &datatype_handles_map,
            function_definitions,
        );

        let signatures = move_module
            .signatures()
            .iter()
            .map(|s| {
                s.0.iter()
                    .map(|t| IntermediateType::try_from_signature_token(t, &datatype_handles_map))
                    .collect::<std::result::Result<Vec<IntermediateType>, _>>()
            })
            .collect::<std::result::Result<Vec<Vec<IntermediateType>>, _>>()
            .unwrap();

        ModuleData {
            constants: move_module.constant_pool.clone(), // TODO: Clone
            functions,
            structs,
            enums,
            signatures,
            datatype_handles_map,
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
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> (
        Vec<(StructDefinitionIndex, Vec<IntermediateType>)>,
        HashMap<FieldInstantiationIndex, usize>,
    ) {
        let mut module_generic_structs_instances = vec![];
        let mut generic_fields_to_struct_map = HashMap::new();

        for (index, struct_instance) in module.struct_instantiations().iter().enumerate() {
            // Map the struct instantiation to the generic struct definition and the instantiation
            // types. The index in the array will match the PackGeneric(index) instruction
            let struct_instantiation_types = module
                .signature_at(struct_instance.type_parameters)
                .0
                .iter()
                .map(|t| IntermediateType::try_from_signature_token(t, datatype_handles_map))
                .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
                .unwrap();

            module_generic_structs_instances
                .push((struct_instance.def, struct_instantiation_types));

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
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> HashMap<FieldInstantiationIndex, (FieldHandleIndex, Vec<IntermediateType>)> {
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
                        .iter()
                        .map(|t| {
                            IntermediateType::try_from_signature_token(t, datatype_handles_map)
                        })
                        .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
                        .unwrap(),
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

    fn process_function_definitions<'move_package>(
        module_id: ModuleId,
        move_module: &'move_package CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        function_definitions: &mut GlobalFunctionTable<'move_package>,
    ) -> FunctionData {
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

        FunctionData {
            arguments: functions_arguments,
            returns: functions_returns,
            calls: function_calls,
            information: function_information,
        }
    }

    pub fn get_signatures_by_index(&self, index: SignatureIndex) -> Result<&Vec<IntermediateType>> {
        self.signatures
            .get(index.into_index())
            .ok_or(CompilationContextError::SignatureNotFound(index))
    }
}
