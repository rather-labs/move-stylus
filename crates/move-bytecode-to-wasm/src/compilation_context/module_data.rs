mod enum_data;
mod function_data;
mod struct_data;

use crate::{
    GlobalFunctionTable,
    compilation_context::reserved_modules::STYLUS_FRAMEWORK_ADDRESS,
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
        Ability, AbilitySet, Constant, DatatypeHandleIndex, EnumDefinitionIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinition, FunctionDefinitionIndex, Signature,
        SignatureIndex, SignatureToken, StructDefInstantiationIndex, StructDefinitionIndex,
        VariantHandleIndex, Visibility,
    },
    internals::ModuleIndex,
};
use move_package::{
    compilation::compiled_package::CompiledUnitWithSource,
    source_package::parsed_manifest::PackageName,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};
use struct_data::StructData;

use super::{CompilationContextError, Result};

#[derive(Debug)]
pub enum UserDefinedType {
    /// Struct defined in this module
    Struct { module_id: ModuleId, index: u16 },

    /// Enum defined in this module
    Enum(usize),
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Address([u8; 32]);

impl Address {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Address(bytes)
    }
}

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

impl Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address[{}]", self)
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

// TODO: This just makes sense for testing
impl Default for ModuleId {
    fn default() -> Self {
        Self {
            address: Address::from([0; 32]),
            module_name: "default".to_owned(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ModuleData {
    /// Module's ID
    pub id: ModuleId,

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
        move_module_dependencies: &'move_package [(PackageName, CompiledUnitWithSource)],
        root_compiled_units: &'move_package [CompiledUnitWithSource],
        function_definitions: &mut GlobalFunctionTable<'move_package>,
    ) -> Self {
        let datatype_handles_map = Self::process_datatype_handles(
            &module_id,
            move_module,
            move_module_dependencies,
            root_compiled_units,
        );

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
            module_id.clone(),
            move_module,
            &datatype_handles_map,
            function_definitions,
            move_module_dependencies,
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
            id: module_id,
            constants: move_module.constant_pool.clone(), // TODO: Clone
            functions,
            structs,
            enums,
            signatures,
            datatype_handles_map,
        }
    }

    fn process_datatype_handles(
        module_id: &ModuleId,
        module: &CompiledModule,
        move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
        root_compiled_units: &[CompiledUnitWithSource],
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
                    datatype_handles_map.insert(
                        idx,
                        UserDefinedType::Struct {
                            module_id: module_id.clone(), // TODO: clone
                            index: position as u16,
                        },
                    );
                } else if let Some(position) =
                    module.enum_defs().iter().position(|e| e.enum_handle == idx)
                {
                    datatype_handles_map.insert(idx, UserDefinedType::Enum(position));
                } else {
                    panic!("datatype handle index {index} not found");
                };
            } else {
                let datatype_module = module.module_handle_at(datatype_handle.module);
                let module_address = module.address_identifier_at(datatype_module.address);
                let module_name = module.identifier_at(datatype_module.name);

                let module_id = ModuleId {
                    address: module_address.into_bytes().into(),
                    module_name: module_name.to_string(),
                };

                // Find the module where the external data is defined, we first look for it in the
                // external packages and if we dont't find it, we look for it in the compile units
                // that belong to our package
                let external_module_source = if let Some(external_module) =
                    &move_module_dependencies.iter().find(|(_, m)| {
                        m.unit.name().as_str() == module_name.as_str()
                            && m.unit.address == *module_address
                    }) {
                    &external_module.1.unit.module
                } else if let Some(external_module) = &root_compiled_units.iter().find(|m| {
                    m.unit.name().as_str() == module_name.as_str()
                        && m.unit.address == *module_address
                }) {
                    &external_module.unit.module
                } else {
                    panic!("could not find dependency {module_id}")
                };

                let external_data_name = module.identifier_at(datatype_handle.name);

                let external_dth_idx = external_module_source
                    .datatype_handles()
                    .iter()
                    .position(|dth| {
                        external_module_source.identifier_at(dth.name) == external_data_name
                    })
                    .unwrap();
                let external_dth_idx = DatatypeHandleIndex::new(external_dth_idx as u16);

                if let Some(position) = external_module_source
                    .struct_defs()
                    .iter()
                    .position(|s| s.struct_handle == external_dth_idx)
                {
                    datatype_handles_map.insert(
                        idx,
                        UserDefinedType::Struct {
                            module_id,
                            index: position as u16,
                        },
                    );
                } else if let Some(position) = module
                    .enum_defs()
                    .iter()
                    .position(|e| e.enum_handle == external_dth_idx)
                {
                    datatype_handles_map.insert(idx, UserDefinedType::Enum(position));
                } else {
                    panic!("datatype handle index {index} not found");
                };
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

            let struct_datatype_handle = module.datatype_handle_at(struct_def.struct_handle);
            let identifier = module
                .identifier_at(struct_datatype_handle.name)
                .to_string();

            let is_saved_in_storage = struct_datatype_handle
                .abilities
                .into_iter()
                .any(|a| a == Ability::Key);

            let is_one_time_witness = Self::is_one_time_witness(module, struct_def.struct_handle);

            module_structs.push(IStruct::new(
                struct_index,
                identifier,
                all_fields,
                fields_map,
                is_saved_in_storage,
                is_one_time_witness,
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
        move_module_dependencies: &'move_package [(PackageName, CompiledUnitWithSource)],
    ) -> FunctionData {
        // Return types of functions in intermediate types. Used to fill the stack type
        let mut functions_returns = Vec::new();
        let mut functions_arguments = Vec::new();
        let mut function_calls = Vec::new();
        let mut function_information = Vec::new();
        let mut init = None;

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
                type_instantiations: None,
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

                let is_init = Self::is_init(
                    &function_id,
                    move_function_arguments,
                    move_function_return,
                    function_def,
                    datatype_handles_map,
                    move_module,
                    move_module_dependencies,
                );

                if is_init {
                    if init.is_some() {
                        panic!("There can be only a single init function per module.");
                    }
                    init = Some(function_id.clone());
                }

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

        let mut generic_function_calls = Vec::new();
        for function in move_module.function_instantiations().iter() {
            let function_handle = move_module.function_handle_at(function.handle);
            let function_name = move_module.identifier_at(function_handle.name).as_str();
            let function_module = move_module.module_handle_at(function_handle.module);
            let function_module_name = move_module.identifier_at(function_module.name).as_str();
            let function_module_address: Address = move_module
                .address_identifier_at(function_module.address)
                .into_bytes()
                .into();

            let type_instantiations = move_module
                .signature_at(function.type_parameters)
                .0
                .iter()
                .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                .collect::<std::result::Result<Vec<IntermediateType>, anyhow::Error>>()
                .unwrap();

            let function_id = FunctionId {
                identifier: function_name.to_string(),
                module_id: ModuleId {
                    address: function_module_address,
                    module_name: function_module_name.to_string(),
                },
                type_instantiations: Some(type_instantiations),
            };

            generic_function_calls.push(function_id);
        }

        FunctionData {
            arguments: functions_arguments,
            returns: functions_returns,
            calls: function_calls,
            generic_calls: generic_function_calls,
            information: function_information,
            init,
        }
    }

    pub fn get_signatures_by_index(&self, index: SignatureIndex) -> Result<&Vec<IntermediateType>> {
        self.signatures
            .get(index.into_index())
            .ok_or(CompilationContextError::SignatureNotFound(index))
    }

    // The init() function is a special function that is called once when the module is first deployed,
    // so it is a good place to put the code that initializes module's objects and sets up the environment and configuration.
    //
    // For the init() function to be considered valid, it must adhere to the following requirements:
    // 1. It must be named `init`.
    // 2. It must be private.
    // 3. It must have &TxContext or &mut TxContext as its last argument, with an optional One Time Witness (OTW) as its first argument.
    // 4. It must not return any values.
    //
    // fun init(ctx: &TxContext) { /* ... */}
    // fun init(otw: OTW, ctx: &mut TxContext) { /* ... */ }
    //

    /// Checks if the given function (by index) is a valid `init` function.
    // TODO: Note that we currently trigger a panic if a function named 'init' fails to satisfy certain criteria to qualify as a constructor.
    // This behavior is not enforced by the move compiler itself.
    fn is_init(
        function_id: &FunctionId,
        move_function_arguments: &Signature,
        move_function_return: &Signature,
        function_def: &FunctionDefinition,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        module: &CompiledModule,
        move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
    ) -> bool {
        // Constants
        const INIT_FUNCTION_NAME: &str = "init";

        // Error messages
        const BAD_ARGS_ERROR_MESSAGE: &str = "invalid arguments";
        const BAD_VISIBILITY_ERROR_MESSAGE: &str = "expected private visibility";
        const BAD_RETURN_ERROR_MESSAGE: &str = "expected no return values";

        // Must be named `init`
        if function_id.identifier != INIT_FUNCTION_NAME {
            return false;
        }

        // Must be private
        assert_eq!(
            function_def.visibility,
            Visibility::Private,
            "{}",
            BAD_VISIBILITY_ERROR_MESSAGE
        );

        // Must have 1 or 2 arguments
        let arg_count = move_function_arguments.len();
        assert!((1..=2).contains(&arg_count), "{}", BAD_ARGS_ERROR_MESSAGE);

        // Check TxContext in the last argument
        let last_arg = move_function_arguments.0.last().map(|last| {
            IntermediateType::try_from_signature_token(last, datatype_handles_map).unwrap()
        });

        // The compilation context is not available yet, so we can't use it to check if the
        // `TxContext` is the one from the stylus framework. It is done manually
        let is_tx_context_ref = match last_arg {
            Some(IntermediateType::IRef(inner)) | Some(IntermediateType::IMutRef(inner)) => {
                match inner.as_ref() {
                    IntermediateType::IStruct {
                        module_id, index, ..
                    } if module_id.module_name == "tx_context"
                        && module_id.address == STYLUS_FRAMEWORK_ADDRESS =>
                    {
                        // TODO: Look for this external module one time and pass it down to this
                        // function
                        let external_module_source = &move_module_dependencies
                            .iter()
                            .find(|(_, m)| {
                                m.unit.name().as_str() == "tx_context"
                                    && Address::from(m.unit.address.into_bytes())
                                        == STYLUS_FRAMEWORK_ADDRESS
                            })
                            .expect("could not find stylus framework as dependency")
                            .1
                            .unit
                            .module;

                        let struct_ = external_module_source
                            .struct_def_at(StructDefinitionIndex::new(*index));
                        let handle =
                            external_module_source.datatype_handle_at(struct_.struct_handle);
                        let identifier = external_module_source.identifier_at(handle.name);
                        identifier.as_str() == "TxContext"
                    }

                    _ => false,
                }
            }
            _ => false,
        };

        assert!(is_tx_context_ref, "{}", BAD_ARGS_ERROR_MESSAGE);

        // Check OTW if 2 arguments
        if arg_count == 2 {
            let SignatureToken::Datatype(idx) = &move_function_arguments.0[0] else {
                panic!("{}", BAD_ARGS_ERROR_MESSAGE);
            };

            assert!(
                Self::is_one_time_witness(module, *idx),
                "{}",
                BAD_ARGS_ERROR_MESSAGE
            );
        }

        // Must not return any values
        assert!(
            move_function_return.is_empty(),
            "{}",
            BAD_RETURN_ERROR_MESSAGE
        );

        true
    }

    /// Checks if the given signature token is a one-time witness type.
    //
    // OTW (One-time witness) types are structs with the following requirements:
    // i. Their name is the upper-case version of the module's name.
    // ii. They have no fields (or a single boolean field).
    // iii. They have no type parameters.
    // iv. They have only the 'drop' ability.
    fn is_one_time_witness(
        module: &CompiledModule,
        datatype_handle_index: DatatypeHandleIndex,
    ) -> bool {
        // 1. Datatype handle must be a struct
        let datatype_handle = module.datatype_handle_at(datatype_handle_index);

        // 2. Name must match uppercase module name
        let module_handle = module.module_handle_at(datatype_handle.module);
        let module_name = module.identifier_at(module_handle.name).as_str();
        let struct_name = module.identifier_at(datatype_handle.name).as_str();
        if struct_name != module_name.to_ascii_uppercase() {
            return false;
        }

        // 3. Must have only the Drop ability
        if datatype_handle.abilities != (AbilitySet::EMPTY | Ability::Drop) {
            return false;
        }

        // 4. Must have no type parameters
        if !datatype_handle.type_parameters.is_empty() {
            return false;
        }

        // 5. Must have 0 or 1 field (and if 1, it must be Bool)
        let struct_def = match module
            .struct_defs
            .iter()
            .find(|def| def.struct_handle == datatype_handle_index)
        {
            Some(def) => def,
            None => return false,
        };

        let field_count = struct_def.declared_field_count().unwrap_or(0);
        if field_count > 1 {
            return false;
        }

        if field_count == 1 {
            let field = struct_def.field(0).unwrap();
            if field.signature.0 != SignatureToken::Bool {
                return false;
            }
        }

        true
    }
}
