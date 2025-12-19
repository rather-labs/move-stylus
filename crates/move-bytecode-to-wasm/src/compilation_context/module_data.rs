pub mod enum_data;
pub mod error;
pub mod function_data;
pub mod struct_data;

use crate::{
    GlobalFunctionTable,
    compilation_context::reserved_modules::STYLUS_FRAMEWORK_ADDRESS,
    hasher::get_hasher,
    translation::{
        functions::MappedFunction,
        intermediate_types::{
            IntermediateType,
            enums::{IEnum, IEnumVariant},
            error::IntermediateTypeError,
            structs::{IStruct, IStructType},
        },
        table::FunctionId,
    },
};
use enum_data::{EnumData, VariantData, VariantInstantiationData};
use error::ModuleDataError;
use function_data::FunctionData;
use move_binary_format::{
    CompiledModule,
    file_format::{
        Ability, AbilitySet, Constant, DatatypeHandleIndex, EnumDefInstantiationIndex,
        EnumDefinitionIndex, FieldHandleIndex, FieldInstantiationIndex, FunctionDefinition,
        FunctionDefinitionIndex, Signature, SignatureIndex, SignatureToken,
        StructDefInstantiationIndex, StructDefinitionIndex, VariantHandleIndex,
        VariantInstantiationHandleIndex, Visibility,
    },
    internals::ModuleIndex,
};
use move_package::{
    compilation::compiled_package::CompiledUnitWithSource,
    source_package::parsed_manifest::PackageName,
};
use move_parse_special_attributes::{
    SpecialAttributes,
    function_modifiers::{Function, FunctionModifier},
};
use move_symbol_pool::Symbol;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};
use struct_data::StructData;

use super::{CompilationContextError, Result, reserved_modules::SF_MODULE_NAME_TX_CONTEXT};

#[derive(Debug)]
pub enum UserDefinedType {
    /// Struct defined in this module
    Struct { module_id: ModuleId, index: u16 },

    /// Enum defined in this module
    Enum { module_id: ModuleId, index: u16 },
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Address([u8; 32]);

impl Address {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Address(bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(last_nonzero) = self.0.iter().rposition(|&b| b != 0) {
            for byte in &self.0[last_nonzero..] {
                write!(f, "0x{byte:02x}")?;
            }
        } else {
            write!(f, "0x0")?;
        }

        Ok(())
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address[{self}]")
    }
}

impl From<[u8; 32]> for Address {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ModuleId {
    pub address: Address,
    pub module_name: Symbol,
}

impl ModuleId {
    pub fn new(address: Address, module_name: &str) -> Self {
        Self {
            address,
            module_name: Symbol::from(module_name),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = get_hasher();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}

impl Hash for ModuleId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.as_slice().hash(state);
        self.module_name.as_str().hash(state);
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.address, self.module_name)
    }
}

#[cfg(test)]
impl Default for ModuleId {
    fn default() -> Self {
        Self {
            address: Address::from([0; 32]),
            module_name: Symbol::from("default"),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
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

    /// Function and struct special attributes for EVM contexts
    pub special_attributes: SpecialAttributes,
}

impl ModuleData {
    pub fn build_module_data<'move_package>(
        module_id: ModuleId,
        move_module: &'move_package CompiledUnitWithSource,
        move_module_dependencies: &'move_package [(PackageName, CompiledUnitWithSource)],
        root_compiled_units: &'move_package [&CompiledUnitWithSource],
        function_definitions: &mut GlobalFunctionTable<'move_package>,
        special_attributes: SpecialAttributes,
    ) -> Result<Self> {
        let move_module_unit = &move_module.unit.module;

        let datatype_handles_map = Self::process_datatype_handles(
            &module_id,
            move_module_unit,
            move_module_dependencies,
            root_compiled_units,
        )?;

        // Module's structs
        let (module_structs, fields_to_struct_map) = Self::process_concrete_structs(
            move_module_unit,
            &datatype_handles_map,
            &special_attributes,
        )?;

        let (module_generic_structs_instances, generic_fields_to_struct_map) =
            Self::process_generic_structs(move_module_unit, &datatype_handles_map)?;

        let instantiated_fields_to_generic_fields =
            Self::process_generic_field_instances(move_module_unit, &datatype_handles_map)?;

        let structs = StructData {
            structs: module_structs,
            generic_structs_instances: module_generic_structs_instances,
            fields_to_struct: fields_to_struct_map,
            generic_fields_to_struct: generic_fields_to_struct_map,
            instantiated_fields_to_generic_fields,
        };

        // Module's enums
        let (module_enums, variants_to_enum_map) =
            Self::process_concrete_enums(move_module_unit, &datatype_handles_map)?;

        let (module_generic_enum_instantiations, variants_instantiation_to_enum_map) =
            Self::process_generic_enums(move_module_unit, &datatype_handles_map)?;

        let enums = EnumData {
            enums: module_enums,
            variants_to_enum: variants_to_enum_map,
            generic_enum_instantiations: module_generic_enum_instantiations,
            variants_instantiation_to_enum: variants_instantiation_to_enum_map,
        };

        let functions = Self::process_function_definitions(
            module_id.clone(),
            move_module_unit,
            &datatype_handles_map,
            function_definitions,
            move_module_dependencies,
            &special_attributes,
        )?;

        let signatures = move_module_unit
            .signatures()
            .iter()
            .map(|s| {
                s.0.iter()
                    .map(|t| IntermediateType::try_from_signature_token(t, &datatype_handles_map))
                    .collect::<std::result::Result<Vec<IntermediateType>, _>>()
            })
            .collect::<std::result::Result<Vec<Vec<IntermediateType>>, _>>()?;

        Ok(ModuleData {
            id: module_id,
            constants: move_module_unit.constant_pool.clone(), // TODO: Clone
            functions,
            structs,
            enums,
            signatures,
            datatype_handles_map,
            special_attributes,
        })
    }

    fn process_datatype_handles(
        module_id: &ModuleId,
        module: &CompiledModule,
        move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
        root_compiled_units: &[&CompiledUnitWithSource],
    ) -> Result<HashMap<DatatypeHandleIndex, UserDefinedType>> {
        let mut datatype_handles_map = HashMap::new();

        for (index, datatype_handle) in module.datatype_handles().iter().enumerate() {
            let idx = DatatypeHandleIndex::new(index as u16);

            // Assert the index we constructed is ok
            if datatype_handle != module.datatype_handle_at(idx) {
                return Err(ModuleDataError::GeneratedInvalidDataTypeHandleIndex(index))?;
            }

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
                    datatype_handles_map.insert(
                        idx,
                        UserDefinedType::Enum {
                            module_id: module_id.clone(),
                            index: position as u16,
                        },
                    );
                } else {
                    return Err(CompilationContextError::DatatypeHanldeIndexNotFound(index));
                };
            } else {
                let datatype_module = module.module_handle_at(datatype_handle.module);
                let module_address = module.address_identifier_at(datatype_module.address);
                let module_name = module.identifier_at(datatype_module.name);

                let module_id =
                    ModuleId::new(module_address.into_bytes().into(), module_name.as_str());

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
                    return Err(CompilationContextError::ModuleNotFound(module_id));
                };

                let external_data_name = module.identifier_at(datatype_handle.name);

                let external_dth_idx = external_module_source
                    .datatype_handles()
                    .iter()
                    .position(|dth| {
                        external_module_source.identifier_at(dth.name) == external_data_name
                    })
                    .ok_or(CompilationContextError::ExternalDatatypeHandlerIndexNotFound)?;
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
                    datatype_handles_map.insert(
                        idx,
                        UserDefinedType::Enum {
                            module_id: module_id.clone(),
                            index: position as u16,
                        },
                    );
                } else {
                    return Err(CompilationContextError::DatatypeHanldeIndexNotFound(index));
                };
            }
        }

        Ok(datatype_handles_map)
    }

    fn process_concrete_structs(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        module_special_attributes: &SpecialAttributes,
    ) -> Result<(
        Vec<IStruct>,
        HashMap<FieldHandleIndex, StructDefinitionIndex>,
    )> {
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
                    )?;

                    let field_index = module
                        .field_handles()
                        .iter()
                        .position(|f| f.field == field_index as u16 && f.owner == struct_index)
                        .map(|i| FieldHandleIndex::new(i as u16));

                    // If field_index is None means the field is never referenced in the code
                    if let Some(field_index) = field_index {
                        let res = fields_map.insert(field_index, intermediate_type.clone());
                        if res.is_some() {
                            return Err(ModuleDataError::FieldAlreadyExists {
                                struct_index: struct_index.into_index(),
                                field_index: field_index.into_index(),
                            })?;
                        }
                        let res = fields_to_struct_map.insert(field_index, struct_index);
                        if res.is_some() {
                            return Err(ModuleDataError::FieldAlreadyMapped {
                                struct_index: struct_index.into_index(),
                                field_index: field_index.into_index(),
                            })?;
                        }
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

            let has_key = struct_datatype_handle
                .abilities
                .into_iter()
                .any(|a| a == Ability::Key);

            let type_ = if Self::is_one_time_witness(module, struct_def.struct_handle) {
                IStructType::OneTimeWitness
            } else if let Some(event) = module_special_attributes.events.get(&identifier) {
                IStructType::Event {
                    indexes: event.indexes,
                    is_anonymous: event.is_anonymous,
                }
            } else if let Some(_abi_error) = module_special_attributes.abi_errors.get(&identifier) {
                IStructType::AbiError
            } else {
                IStructType::Common
            };

            module_structs.push(IStruct::new(
                struct_index,
                identifier,
                all_fields,
                fields_map,
                has_key,
                type_,
            ));
        }

        Ok((module_structs, fields_to_struct_map))
    }

    #[allow(clippy::type_complexity)]
    fn process_generic_structs(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<(
        Vec<(StructDefinitionIndex, Vec<IntermediateType>)>,
        HashMap<FieldInstantiationIndex, usize>,
    )> {
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
                .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>()?;

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

                        if res.is_some() {
                            return Err(ModuleDataError::FieldAlreadyMapped {
                                struct_index: struct_index.into_index(),
                                field_index: generic_field_index.into_index(),
                            })?;
                        }
                    }
                }
            }
        }

        Ok((
            module_generic_structs_instances,
            generic_fields_to_struct_map,
        ))
    }

    fn process_generic_field_instances(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<HashMap<FieldInstantiationIndex, (FieldHandleIndex, Vec<IntermediateType>)>> {
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
                        .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>()?
                ),
            );
        }

        Ok(instantiated_fields_to_generic_fields)
    }

    pub fn process_concrete_enums(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<(Vec<IEnum>, HashMap<VariantHandleIndex, VariantData>)> {
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
                    .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>(
                    )?;

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

                    if res.is_some() {
                        return Err(ModuleDataError::VariantAlreadyExists {
                            enum_index: enum_index.into_index(),
                            variant_index,
                        })?;
                    }
                }
            }

            let enum_datatype_handle = module.datatype_handle_at(enum_def.enum_handle);
            let identifier = module.identifier_at(enum_datatype_handle.name);
            module_enums.push(IEnum::new(identifier.to_string(), index as u16, variants)?);
        }

        Ok((module_enums, variants_to_enum_map))
    }

    #[allow(clippy::type_complexity)]
    pub fn process_generic_enums(
        module: &CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<(
        Vec<(EnumDefinitionIndex, Vec<IntermediateType>)>,
        HashMap<VariantInstantiationHandleIndex, VariantInstantiationData>,
    )> {
        let mut module_generic_enums_instances = vec![];
        let mut variants_instantiation_to_enum_map = HashMap::new();

        for (index, enum_def_instantiation) in module.enum_instantiations().iter().enumerate() {
            let enum_def_index = enum_def_instantiation.def;
            let enum_definition = &module.enum_defs()[enum_def_index.into_index()];

            let enum_instantiation_types = module
                .signature_at(enum_def_instantiation.type_parameters)
                .0
                .iter()
                .map(|t| IntermediateType::try_from_signature_token(t, datatype_handles_map))
                .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>()?;

            module_generic_enums_instances.push((enum_def_index, enum_instantiation_types.clone()));

            // Process all variant instantiation handles for this enum instantiation
            for (variant_instantiation_handle_index, variant_instantiation_handle) in module
                .variant_instantiation_handles()
                .iter()
                .enumerate()
                .filter(|(_idx, v)| v.enum_def.into_index() == index)
            {
                let variant_index = variant_instantiation_handle.variant; // index inside the enum definition
                let variant_definition = &enum_definition.variants[variant_index as usize];

                // Get the types for this specific variant by resolving the variant's field types
                // using the enum's type parameters
                let variant_types = variant_definition
                    .fields
                    .iter()
                    .map(|field| {
                        // Resolve the field's type signature using the enum's type parameters
                        match &field.signature.0 {
                            SignatureToken::TypeParameter(param_idx) => {
                                // This field uses one of the enum's type parameters
                                Ok(enum_instantiation_types[*param_idx as usize].clone())
                            }
                            other_type => {
                                // This field has a concrete type, resolve it normally
                                IntermediateType::try_from_signature_token(
                                    other_type,
                                    datatype_handles_map,
                                )
                            }
                        }
                    })
                    .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>(
                    )?;

                variants_instantiation_to_enum_map.insert(
                    VariantInstantiationHandleIndex::new(variant_instantiation_handle_index as u16),
                    VariantInstantiationData {
                        enum_index: enum_def_index.into_index(),
                        enum_def_instantiation_index: EnumDefInstantiationIndex::new(index as u16),
                        index_inside_enum: variant_index as usize,
                        types: variant_types,
                    },
                );
            }
        }

        Ok((
            module_generic_enums_instances,
            variants_instantiation_to_enum_map,
        ))
    }

    fn process_function_definitions<'move_package>(
        module_id: ModuleId,
        move_module: &'move_package CompiledModule,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        function_definitions: &mut GlobalFunctionTable<'move_package>,
        move_module_dependencies: &'move_package [(PackageName, CompiledUnitWithSource)],
        special_attributes: &SpecialAttributes,
    ) -> Result<FunctionData> {
        // Return types of functions in intermediate types. Used to fill the stack type
        let mut functions_returns = Vec::new();
        let mut functions_arguments = Vec::new();
        let mut function_calls = Vec::new();
        let mut function_information = Vec::new();

        // Special reserved functions
        let mut init: Option<FunctionId> = None;
        let mut receive: Option<FunctionId> = None;
        let mut fallback: Option<FunctionId> = None;

        for (index, function) in move_module.function_handles().iter().enumerate() {
            let move_function_arguments = &move_module.signature_at(function.parameters);

            functions_arguments.push(
                move_function_arguments
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                    .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>(
                    )?,
            );

            let move_function_return = &move_module.signature_at(function.return_);

            functions_returns.push(
                move_function_return
                    .0
                    .iter()
                    .map(|s| IntermediateType::try_from_signature_token(s, datatype_handles_map))
                    .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>(
                    )?,
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
                module_id: ModuleId::new(function_module_address, function_module_name),
                type_instantiations: None,
            };

            // If we find this function, means the module is compiled in test mode. This function
            // is never called, it is just there to pollute the module so it can't be linked in
            // production mode
            if function_name == "unit_test_poison" {
                let function_def =
                    move_module.function_def_at(FunctionDefinitionIndex::new(index as u16));

                function_information.push(MappedFunction::new(
                    function_id.clone(),
                    move_function_arguments,
                    move_function_return,
                    &[],
                    function_def,
                    datatype_handles_map,
                )?);

                function_definitions.insert(function_id.clone(), function_def);

                function_calls.push(function_id);
                continue;
            }

            // If the functions is defined in this module, we can obtain its definition and process
            // it.
            // If the function is not defined here, it will be processed when processing the
            // dependency
            if *function_module_name == *module_id.module_name
                && function_module_address == module_id.address
            {
                let function_def =
                    move_module.function_def_at(FunctionDefinitionIndex::new(index as u16));

                if !function_def.acquires_global_resources.is_empty() {
                    return Err(ModuleDataError::AcquiresGlobalResourceNotEmpty)?;
                }

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
                )?;

                if is_init && init.replace(function_id.clone()).is_some() {
                    return Err(CompilationContextError::DuplicateInitFunction);
                }

                let function_sa = special_attributes
                    .functions
                    .iter()
                    .find(|f| f.name == function_name)
                    .or_else(|| special_attributes.external_calls.get(function_name))
                    .ok_or(ModuleDataError::FunctionByIdentifierNotFound(
                        function_name.to_string(),
                    ))?;

                let is_receive = Self::is_receive(
                    &function_id,
                    move_function_arguments,
                    move_function_return,
                    function_def,
                    function_sa,
                    datatype_handles_map,
                    move_module_dependencies,
                )?;

                if is_receive && receive.replace(function_id.clone()).is_some() {
                    return Err(CompilationContextError::DuplicateReceiveFunction);
                }

                let is_fallback = Self::is_fallback(
                    &function_id,
                    move_function_arguments,
                    move_function_return,
                    function_def,
                    datatype_handles_map,
                    move_module_dependencies,
                )?;

                if is_fallback && fallback.replace(function_id.clone()).is_some() {
                    return Err(CompilationContextError::DuplicateFallbackFunction);
                }

                function_information.push(MappedFunction::new(
                    function_id.clone(),
                    move_function_arguments,
                    move_function_return,
                    code_locals,
                    function_def,
                    datatype_handles_map,
                )?);

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
                .collect::<std::result::Result<Vec<IntermediateType>, IntermediateTypeError>>()?;

            let function_id = FunctionId {
                identifier: function_name.to_string(),
                module_id: ModuleId::new(function_module_address, function_module_name),
                type_instantiations: Some(type_instantiations),
            };

            generic_function_calls.push(function_id);
        }

        Ok(FunctionData {
            arguments: functions_arguments,
            returns: functions_returns,
            calls: function_calls,
            generic_calls: generic_function_calls,
            information: function_information,
            init,
            receive,
            fallback,
        })
    }

    pub fn get_signatures_by_index(&self, index: SignatureIndex) -> Result<&Vec<IntermediateType>> {
        self.signatures
            .get(index.into_index())
            .ok_or(CompilationContextError::SignatureNotFound(index))
    }

    /// Returns `true` if the function is used as a cross contract call
    pub fn is_external_call(&self, function_identifier: &str) -> bool {
        self.special_attributes
            .external_calls
            .contains_key(function_identifier)
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
    // entry fun init(ctx: &TxContext) { /* ... */}
    // entry fun init(otw: OTW, ctx: &mut TxContext) { /* ... */ }
    //

    /// Checks if the given function (by index) is a valid `init` function.
    // This behavior is not enforced by the move compiler itself.
    fn is_init(
        function_id: &FunctionId,
        move_function_arguments: &Signature,
        move_function_return: &Signature,
        function_def: &FunctionDefinition,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        module: &CompiledModule,
        move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
    ) -> Result<bool> {
        // Constants
        const INIT_FUNCTION_NAME: &str = "init";

        // Must be named `init`
        if function_id.identifier != INIT_FUNCTION_NAME {
            return Ok(false);
        }

        if function_def.visibility != Visibility::Private {
            return Err(CompilationContextError::InitFunctionBadPrivacy);
        }

        // Must have 1 or 2 arguments
        let arg_count = move_function_arguments.len();
        if arg_count > 2 {
            return Err(CompilationContextError::InitFunctionTooManyArgs);
        } else if arg_count == 0 {
            return Err(CompilationContextError::InitFunctionNoAguments);
        }

        // Check TxContext in the last argument
        // The compilation context is not available yet, so we can't use it to check if the
        // `TxContext` is the one from the stylus framework. It is done manually
        let is_tx_context_ref = move_function_arguments
            .0
            .last()
            .and_then(|last| {
                IntermediateType::try_from_signature_token(last, datatype_handles_map).ok()
            })
            .is_some_and(|arg| is_tx_context_ref(&arg, move_module_dependencies));

        if !is_tx_context_ref {
            return Err(CompilationContextError::InitFunctionNoTxContext);
        }

        // Check OTW if 2 arguments
        if arg_count == 2 {
            let SignatureToken::Datatype(idx) = &move_function_arguments.0[0] else {
                return Err(CompilationContextError::InitFunctionNoOTW);
            };

            if !Self::is_one_time_witness(module, *idx) {
                return Err(CompilationContextError::InitFunctionNoOTW);
            }
        }

        // Must not return any values
        if !move_function_return.is_empty() {
            return Err(CompilationContextError::InitFunctionBadRetrunValues);
        }

        Ok(true)
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

        if let Some(field) = struct_def.field(0) {
            field.signature.0 == SignatureToken::Bool
        } else {
            true
        }
    }

    // Determines whether a function is a valid receive function.
    // Returns true if the function is a valid receive function, otherwise returns false.
    // If the function name is "Receive" but does not fulfill the requirements, returns an error.
    fn is_receive(
        function_id: &FunctionId,
        move_function_arguments: &Signature,
        move_function_return: &Signature,
        function_def: &FunctionDefinition,
        function_sa: &Function,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
    ) -> Result<bool> {
        // Receive function definition (see https://docs.soliditylang.org/en/latest/contracts.html#receive-ether-function):
        // - A contract can have at most one receive function, declared using receive() external payable { ... } (without the function keyword).
        // - This function cannot have arguments other than a TxContext parameter, cannot return anything and must have external visibility and payable state mutability.
        const RECEIVE_FUNCTION_NAME: &str = "receive";

        // Must be named `receive`. Otherwise, it is not a receive function.
        if function_id.identifier != RECEIVE_FUNCTION_NAME {
            return Ok(false);
        }

        // Since the function is named `receive`, we must verify that it satisfies all specified constraints.
        // If any requirement is not fulfilled, an error will be returned; otherwise return true.

        // Must have external visibility, i.e. it must be entry
        if !function_def.is_entry {
            return Err(CompilationContextError::ReceiveFunctionBadVisibility);
        }

        // The function can only take a reference to the TxContext as its only argument.
        if move_function_arguments.len() > 1 {
            return Err(CompilationContextError::ReceiveFunctionTooManyArguments);
        }
        if let Some(first_arg) = move_function_arguments.0.first() {
            if let Ok(arg) =
                IntermediateType::try_from_signature_token(first_arg, datatype_handles_map)
            {
                if !is_tx_context_ref(&arg, move_module_dependencies) {
                    return Err(CompilationContextError::ReceiveFunctionNonTxContextArgument);
                }
            }
        }

        // Must have no return values
        if !move_function_return.is_empty() {
            return Err(CompilationContextError::ReceiveFunctionHasReturns);
        }

        // Must be payable
        if !function_sa.modifiers.contains(&FunctionModifier::Payable) {
            return Err(CompilationContextError::ReceiveFunctionIsNotPayable);
        }

        Ok(true)
    }

    // Determines whether a function is a valid fallback function.
    // Returns true if the function is a valid fallback function, otherwise returns false.
    // If the function name is "Fallback" but does not fulfill the requirements, returns an error.
    fn is_fallback(
        function_id: &FunctionId,
        move_function_arguments: &Signature,
        move_function_return: &Signature,
        function_def: &FunctionDefinition,
        datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
    ) -> Result<bool> {
        // Fallback function definition (see https://docs.soliditylang.org/en/latest/contracts.html#fallback-function):
        // A contract can have at most one fallback function, declared using either fallback () external [payable]
        // or fallback (bytes calldata input) external [payable] returns (bytes memory output) (both without the function keyword).
        // This function must have external visibility, but it is not required to be payable.
        // A fallback function can be virtual, can override and can have modifiers.

        const FALLBACK_FUNCTION_NAME: &str = "fallback";

        // Must be named `fallback`. Otherwise, it is not a fallback function.
        if function_id.identifier != FALLBACK_FUNCTION_NAME {
            return Ok(false);
        }

        // Since the function is named `fallback`, we must verify that it satisfies all specified constraints.
        // If any requirement is not fulfilled, an error will be returned; otherwise return true.

        // Must have external visibility, i.e. it must be entry
        if !function_def.is_entry {
            return Err(CompilationContextError::FallbackFunctionBadVisibility);
        }

        // Validate arguments based on count
        match move_function_arguments.len() {
            0 => {
                // No arguments: valid
            }
            1 => {
                // 1 argument: must be either vector<u8> or TxContext
                let first_arg = move_function_arguments
                    .0
                    .first()
                    .and_then(|arg| {
                        IntermediateType::try_from_signature_token(arg, datatype_handles_map).ok()
                    })
                    .ok_or(CompilationContextError::FallbackFunctionInvalidArgumentType(1))?;

                // Check if it's fallback calldata or TxContext
                if !(is_fallback_calldata(&first_arg, move_module_dependencies)
                    || is_tx_context_ref(&first_arg, move_module_dependencies))
                {
                    return Err(CompilationContextError::FallbackFunctionInvalidArgumentType(1));
                }
            }
            2 => {
                // 2 arguments: first must be vector<u8>, second must be TxContext
                let first_arg = move_function_arguments
                    .0
                    .first()
                    .and_then(|arg| {
                        IntermediateType::try_from_signature_token(arg, datatype_handles_map).ok()
                    })
                    .ok_or(CompilationContextError::FallbackFunctionInvalidArgumentType(1))?;

                let second_arg = move_function_arguments
                    .0
                    .last()
                    .and_then(|arg| {
                        IntermediateType::try_from_signature_token(arg, datatype_handles_map).ok()
                    })
                    .ok_or(CompilationContextError::FallbackFunctionInvalidArgumentType(2))?;

                // First argument must be fallback calldata
                if !is_fallback_calldata(&first_arg, move_module_dependencies) {
                    return Err(CompilationContextError::FallbackFunctionInvalidArgumentType(1));
                }

                // Second argument must be TxContext
                if !is_tx_context_ref(&second_arg, move_module_dependencies) {
                    return Err(CompilationContextError::FallbackFunctionInvalidArgumentType(2));
                }
            }
            _ => {
                // Already checked above, but this is unreachable
                return Err(CompilationContextError::FallbackFunctionTooManyArguments);
            }
        }

        // Validate return type: must be either empty or a single vector<u8>
        match move_function_return.len() {
            0 => {
                // No return values: valid
            }
            1 => {
                // 1 return value: must be fallback calldata
                let return_type = move_function_return
                    .0
                    .first()
                    .and_then(|arg| {
                        IntermediateType::try_from_signature_token(arg, datatype_handles_map).ok()
                    })
                    .ok_or(CompilationContextError::FallbackFunctionInvalidReturnType)?;

                if !is_fallback_calldata(&return_type, move_module_dependencies) {
                    return Err(CompilationContextError::FallbackFunctionInvalidReturnType);
                }
            }
            _ => {
                // More than 1 return value: invalid
                return Err(CompilationContextError::FallbackFunctionInvalidReturnType);
            }
        }

        Ok(true)
    }
}

fn is_fallback_calldata(
    argument: &IntermediateType,
    move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
) -> bool {
    match argument {
        IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
            match inner.as_ref() {
                IntermediateType::IStruct {
                    module_id, index, ..
                } if module_id.module_name.as_str() == "fallback"
                    && module_id.address == STYLUS_FRAMEWORK_ADDRESS =>
                {
                    // TODO: Look for this external module one time and pass it down to this
                    // function
                    let external_module_source = &move_module_dependencies
                        .iter()
                        .find(|(_, m)| {
                            m.unit.name().as_str() == "fallback"
                                && Address::from(m.unit.address.into_bytes())
                                    == STYLUS_FRAMEWORK_ADDRESS
                        })
                        .expect("could not find stylus framework as dependency")
                        .1
                        .unit
                        .module;

                    let struct_ =
                        external_module_source.struct_def_at(StructDefinitionIndex::new(*index));
                    let handle = external_module_source.datatype_handle_at(struct_.struct_handle);
                    let identifier = external_module_source.identifier_at(handle.name);
                    identifier.as_str() == "Calldata"
                }

                _ => false,
            }
        }
        _ => false,
    }
}

/// Helper function to check if the argument is a reference to the TxContext.
fn is_tx_context_ref(
    argument: &IntermediateType,
    move_module_dependencies: &[(PackageName, CompiledUnitWithSource)],
) -> bool {
    match argument {
        IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner) => {
            match inner.as_ref() {
                IntermediateType::IStruct {
                    module_id, index, ..
                } if module_id.module_name.as_str() == SF_MODULE_NAME_TX_CONTEXT
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

                    let struct_ =
                        external_module_source.struct_def_at(StructDefinitionIndex::new(*index));
                    let handle = external_module_source.datatype_handle_at(struct_.struct_handle);
                    let identifier = external_module_source.identifier_at(handle.name);
                    identifier.as_str() == "TxContext"
                }

                _ => false,
            }
        }
        _ => false,
    }
}
