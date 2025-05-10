// SPDX-License-Identifier: MIT
//
// Copyright (c) 2021, Johannes Stoelp <dev@memzero.de>

use llvm_sys::{core::LLVMGetBasicBlockParent, prelude::LLVMBasicBlockRef};

use std::marker::PhantomData;

use super::FnValue;

/// Wrapper for a LLVM Basic Block.
#[derive(Copy, Clone)]
pub struct BasicBlock<'llvm>(LLVMBasicBlockRef, PhantomData<&'llvm ()>);

impl<'llvm> BasicBlock<'llvm> {
    /// Create a new BasicBlock instance.
    ///
    /// # Panics
    ///
    /// Panics if `bb_ref` is a null pointer.
    pub(super) fn new(bb_ref: LLVMBasicBlockRef) -> BasicBlock<'llvm> {
        assert!(!bb_ref.is_null());
        BasicBlock(bb_ref, PhantomData)
    }

    /// Get the raw LLVM value reference.
    #[inline]
    pub(super) fn bb_ref(&self) -> LLVMBasicBlockRef {
        self.0
    }

    /// Get the function to which the basic block belongs.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn get_parent(&self) -> FnValue<'llvm> {
        let value_ref = unsafe { LLVMGetBasicBlockParent(self.bb_ref()) };
        assert!(!value_ref.is_null());

        FnValue::new(value_ref)
    }
}
