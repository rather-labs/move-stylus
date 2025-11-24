#[derive(Debug, thiserror::Error)]
pub enum ModuleDataError {
    #[error(
        "there was an error creating a field in struct {struct_index}, field with index {field_index} already exist"
    )]
    FieldAlreadyExists {
        struct_index: usize,
        field_index: usize,
    },

    #[error(
        "there was an error mapping field {field_index} to struct {struct_index}, already mapped"
    )]
    FieldAlreadyMapped {
        struct_index: usize,
        field_index: usize,
    },

    #[error(
        "there was an error creating a variant in struct {variant_index}, variant with index {variant_index} already exist"
    )]
    VariantAlreadyExists {
        enum_index: usize,
        variant_index: usize,
    },

    #[error("acquires global resource is not empty")]
    AcquiresGlobalResourceNotEmpty,
}
