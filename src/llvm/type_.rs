use llvm_sys::{
    core::{LLVMConstReal, LLVMDumpType, LLVMGetTypeKind},
    prelude::LLVMTypeRef,
    LLVMTypeKind,
};

use std::marker::PhantomData;

use super::Value;

/// Wrapper for a LLVM Type Reference.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Type<'llvm>(LLVMTypeRef, PhantomData<&'llvm ()>);

impl<'llvm> Type<'llvm> {
    /// Create a new Type instance.
    ///
    /// # Panics
    ///
    /// Panics if `type_ref` is a null pointer.
    pub(super) fn new(type_ref: LLVMTypeRef) -> Self {
        assert!(!type_ref.is_null());
        Type(type_ref, PhantomData)
    }

    /// Get the raw LLVM type reference.
    #[inline]
    pub(super) fn type_ref(&self) -> LLVMTypeRef {
        self.0
    }

    /// Get the LLVM type kind for the given type reference.
    pub(super) fn kind(&self) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(self.type_ref()) }
    }

    /// Dump the LLVM Type to stdout.
    pub fn dump(&self) {
        unsafe { LLVMDumpType(self.type_ref()) };
    }

    /// Get a value reference representing the const `f64` value.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn const_f64(self, n: f64) -> Value<'llvm> {
        debug_assert_eq!(
            self.kind(),
            LLVMTypeKind::LLVMDoubleTypeKind,
            "Expected a double type when creating const f64 value!"
        );

        let value_ref = unsafe { LLVMConstReal(self.type_ref(), n) };
        Value::new(value_ref)
    }
}
