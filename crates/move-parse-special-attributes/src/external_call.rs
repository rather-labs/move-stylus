//! This module is in charge if checking all the constraints related to marking a function as an
//! external call.

pub mod error;
pub mod external_struct;
mod function;
mod structs;

pub(crate) use function::validate_external_call_function;
pub(crate) use structs::validate_external_call_struct;
