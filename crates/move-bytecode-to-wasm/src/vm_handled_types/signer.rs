// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use super::{VmHandledType, error::VmHandledTypeError};
use crate::{CompilationContext, compilation_context::ModuleId, runtime::RuntimeFunction};
use walrus::{InstrSeqBuilder, Module};

pub struct Signer;

impl VmHandledType for Signer {
    const IDENTIFIER: &str = "signer";

    fn inject(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), VmHandledTypeError> {
        let inject_signer_fn =
            RuntimeFunction::InjectSigner.get(module, Some(compilation_ctx), None)?;

        block.call(inject_signer_fn);
        Ok(())
    }

    fn is_vm_type(
        _module_id: &ModuleId,
        _index: u16,
        _compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        Ok(true)
    }
}
