#![allow(unused)]

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction},
    core::{
        LLVMAddIncoming, LLVMAppendExistingBasicBlock, LLVMCountBasicBlocks, LLVMCountParams,
        LLVMDumpValue, LLVMGetParam, LLVMGetReturnType, LLVMGetValueKind, LLVMGetValueName2,
        LLVMIsAFunction, LLVMIsAPHINode, LLVMSetValueName2, LLVMTypeOf,
    },
    prelude::LLVMValueRef,
    LLVMTypeKind, LLVMValueKind,
};

use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::Deref;

use super::BasicBlock;
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

    /// Check if value is `function` type.
    pub(super) fn is_function(&self) -> bool {
        let cast = unsafe { LLVMIsAFunction(self.value_ref()) };
        !cast.is_null()
    }

    /// Check if value is `phinode` type.
    pub(super) fn is_phinode(&self) -> bool {
        let cast = unsafe { LLVMIsAPHINode(self.value_ref()) };
        !cast.is_null()
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
        debug_assert!(
            value.is_function(),
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

    /// Append a Basic Block to the end of the function value.
    pub fn append_basic_block(&self, bb: BasicBlock<'llvm>) {
        unsafe {
            LLVMAppendExistingBasicBlock(self.value_ref(), bb.bb_ref());
        }
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

/// Wrapper for a LLVM Value Reference specialized for contexts where phi values are needed.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PhiValue<'llvm>(Value<'llvm>);

impl<'llvm> Deref for PhiValue<'llvm> {
    type Target = Value<'llvm>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'llvm> PhiValue<'llvm> {
    /// Create a new PhiValue instance.
    ///
    /// # Panics
    ///
    /// Panics if `value_ref` is a null pointer.
    pub(super) fn new(value_ref: LLVMValueRef) -> Self {
        let value = Value::new(value_ref);
        debug_assert!(
            value.is_phinode(),
            "Expected a phinode value when constructing PhiValue!"
        );

        PhiValue(value)
    }

    /// Add an incoming value to the end of a PHI list.
    pub fn add_incoming(&self, ival: Value<'llvm>, ibb: BasicBlock<'llvm>) {
        debug_assert_eq!(
            ival.type_of().kind(),
            self.type_of().kind(),
            "Type of incoming phi value must be the same as the type used to build the phi node."
        );

        unsafe {
            LLVMAddIncoming(
                self.value_ref(),
                &mut ival.value_ref() as _,
                &mut ibb.bb_ref() as _,
                1,
            );
        }
    }
}
