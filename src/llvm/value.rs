use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction},
    core::{
        LLVMCountBasicBlocks, LLVMCountParams, LLVMDumpValue, LLVMGetParam, LLVMGetReturnType,
        LLVMGetValueKind, LLVMGetValueName2, LLVMSetValueName2, LLVMTypeOf,
    },
    prelude::LLVMValueRef,
    LLVMTypeKind, LLVMValueKind,
};

use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::Deref;

use super::Type;

/// Wrapper for a LLVM Value Reference.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Value<'llvm>(LLVMValueRef, PhantomData<&'llvm ()>);

impl<'llvm> Value<'llvm> {
    /// Create a new Value instance.
    ///
    /// # Panics
    ///
    /// Panics if `value_ref` is a null pointer.
    pub(super) fn new(value_ref: LLVMValueRef) -> Self {
        assert!(!value_ref.is_null());
        Value(value_ref, PhantomData)
    }

    /// Get the raw LLVM value reference.
    #[inline]
    pub(super) fn value_ref(&self) -> LLVMValueRef {
        self.0
    }

    /// Get the LLVM value kind for the given value reference.
    pub(super) fn kind(&self) -> LLVMValueKind {
        unsafe { LLVMGetValueKind(self.value_ref()) }
    }

    /// Dump the LLVM Value to stdout.
    pub fn dump(&self) {
        unsafe { LLVMDumpValue(self.value_ref()) };
    }

    /// Get a type reference representing for the given value reference.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn type_of(&self) -> Type<'llvm> {
        let type_ref = unsafe { LLVMTypeOf(self.value_ref()) };
        Type::new(type_ref)
    }

    /// Set the name for the given value reference.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn set_name(&self, name: &str) {
        unsafe { LLVMSetValueName2(self.value_ref(), name.as_ptr().cast(), name.len()) };
    }

    /// Get the name for the given value reference.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn get_name(&self) -> &'llvm str {
        let name = unsafe {
            let mut len: libc::size_t = 0;
            let name = LLVMGetValueName2(self.0, &mut len as _);
            assert!(!name.is_null());

            CStr::from_ptr(name)
        };

        // TODO: Does this string live for the time of the LLVM context?!
        name.to_str()
            .expect("Expected valid UTF8 string from LLVM API")
    }

    /// Check if value is of `f64` type.
    pub fn is_f64(&self) -> bool {
        self.type_of().kind() == LLVMTypeKind::LLVMDoubleTypeKind
    }

    /// Check if value is of integer type.
    pub fn is_int(&self) -> bool {
        self.type_of().kind() == LLVMTypeKind::LLVMIntegerTypeKind
    }
}

/// Wrapper for a LLVM Value Reference specialized for contexts where function values are needed.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct FnValue<'llvm>(Value<'llvm>);

impl<'llvm> Deref for FnValue<'llvm> {
    type Target = Value<'llvm>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'llvm> FnValue<'llvm> {
    /// Create a new FnValue instance.
    ///
    /// # Panics
    ///
    /// Panics if `value_ref` is a null pointer.
    pub(super) fn new(value_ref: LLVMValueRef) -> Self {
        let value = Value::new(value_ref);
        debug_assert_eq!(
            value.kind(),
            LLVMValueKind::LLVMFunctionValueKind,
            "Expected a fn value when constructing FnValue!"
        );

        FnValue(value)
    }

    /// Get a type reference representing the return value of the given function value.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn ret_type(&self) -> Type<'llvm> {
        let type_ref = unsafe { LLVMGetReturnType(LLVMTypeOf(self.value_ref())) };
        Type::new(type_ref)
    }

    /// Get the number of function arguments for the given function value.
    pub fn args(&self) -> usize {
        unsafe { LLVMCountParams(self.value_ref()) as usize }
    }

    /// Get a value reference for the function argument at index `idx`.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer or indexed out of bounds.
    pub fn arg(&self, idx: usize) -> Value<'llvm> {
        assert!(idx < self.args());

        let value_ref = unsafe { LLVMGetParam(self.value_ref(), idx as libc::c_uint) };
        Value::new(value_ref)
    }

    /// Get the number of Basic Blocks for the given function value.
    pub fn basic_blocks(&self) -> usize {
        unsafe { LLVMCountBasicBlocks(self.value_ref()) as usize }
    }

    /// Verify that the given function is valid.
    pub fn verify(&self) -> bool {
        unsafe {
            LLVMVerifyFunction(
                self.value_ref(),
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
            ) == 0
        }
    }
}
