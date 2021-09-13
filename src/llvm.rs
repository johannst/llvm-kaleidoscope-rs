//! Safe wrapper around the LLVM C API.
//!
//! References returned from the LLVM API are tied to the `'llvm` lifetime which is bound to the
//! context where the objects are created in.
//! We do not offer wrappers to remove or delete any objects in the context and therefore all the
//! references will be valid for the liftime of the context.

use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction};
use llvm_sys::core::{
    LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFMul,
    LLVMBuildFSub, LLVMBuildRet, LLVMBuildUIToFP, LLVMConstReal, LLVMContextCreate,
    LLVMContextDispose, LLVMCountBasicBlocks, LLVMCountParams, LLVMCreateBuilderInContext,
    LLVMDisposeBuilder, LLVMDisposeModule, LLVMDoubleTypeInContext, LLVMDumpModule, LLVMDumpType,
    LLVMDumpValue, LLVMGetNamedFunction, LLVMGetParam, LLVMGetReturnType, LLVMGetTypeKind,
    LLVMGetValueKind, LLVMGetValueName2, LLVMModuleCreateWithNameInContext,
    LLVMPositionBuilderAtEnd, LLVMSetValueName2, LLVMTypeOf,
};
use llvm_sys::prelude::{
    LLVMBasicBlockRef, LLVMBool, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef,
    LLVMValueRef,
};
use llvm_sys::{LLVMRealPredicate, LLVMTypeKind, LLVMValueKind};

use std::convert::TryFrom;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::Deref;

use crate::SmallCStr;

// Definition of LLVM C API functions using our `repr(transparent)` types.
extern "C" {
    fn LLVMFunctionType(
        ReturnType: Type<'_>,
        ParamTypes: *mut Type<'_>,
        ParamCount: ::libc::c_uint,
        IsVarArg: LLVMBool,
    ) -> LLVMTypeRef;
    fn LLVMBuildCall2(
        arg1: LLVMBuilderRef,
        arg2: Type<'_>,
        Fn: FnValue<'_>,
        Args: *mut Value<'_>,
        NumArgs: ::libc::c_uint,
        Name: *const ::libc::c_char,
    ) -> LLVMValueRef;
}

// ====================
//   Module / Context
// ====================

/// Wrapper for a LLVM Module with its own LLVM Context.
pub struct Module {
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
}

impl<'llvm> Module {
    /// Create a new Module instance.
    ///
    /// # Panics
    ///
    /// Panics if creating the context or the module fails.
    pub fn new() -> Self {
        let (ctx, module) = unsafe {
            let c = LLVMContextCreate();
            let m = LLVMModuleCreateWithNameInContext(b"module\0".as_ptr().cast(), c);
            assert!(!c.is_null() && !m.is_null());
            (c, m)
        };

        Module { ctx, module }
    }

    /// Dump LLVM IR emitted into the Module to stdout.
    pub fn dump(&self) {
        unsafe { LLVMDumpModule(self.module) };
    }

    /// Get a type reference representing a `f64` float.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn type_f64(&self) -> Type<'llvm> {
        let type_ref = unsafe { LLVMDoubleTypeInContext(self.ctx) };
        Type::new(type_ref)
    }

    /// Get a type reference representing a `fn(args) -> ret` function.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn type_fn(&'llvm self, args: &mut [Type<'llvm>], ret: Type<'llvm>) -> Type<'llvm> {
        let type_ref = unsafe {
            LLVMFunctionType(
                ret,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                0, /* IsVarArg */
            )
        };
        Type::new(type_ref)
    }

    /// Add a function with the given `name` and `fn_type` to the module and return a value
    /// reference representing the function.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer or `name` could not be converted to a
    /// [`SmallCStr`].
    pub fn add_fn(&'llvm self, name: &str, fn_type: Type<'llvm>) -> FnValue<'llvm> {
        debug_assert_eq!(
            fn_type.kind(),
            LLVMTypeKind::LLVMFunctionTypeKind,
            "Expected a function type when adding a function!"
        );

        let name = SmallCStr::try_from(name)
            .expect("Failed to convert 'name' argument to small C string!");

        let value_ref = unsafe { LLVMAddFunction(self.module, name.as_ptr(), fn_type.0) };
        FnValue::new(value_ref)
    }

    /// Get a function value reference to the function with the given `name` if it was previously
    /// added to the module with [`add_fn`][Module::add_fn].
    ///
    /// # Panics
    ///
    /// Panics if `name` could not be converted to a [`SmallCStr`].
    pub fn get_fn(&'llvm self, name: &str) -> Option<FnValue<'llvm>> {
        let name = SmallCStr::try_from(name)
            .expect("Failed to convert 'name' argument to small C string!");

        let value_ref = unsafe { LLVMGetNamedFunction(self.module, name.as_ptr()) };

        (!value_ref.is_null()).then(|| FnValue::new(value_ref))
    }

    /// Append a Basic Block to the end of the function referenced by the value reference
    /// `fn_value`.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn append_basic_block(&'llvm self, fn_value: FnValue<'llvm>) -> BasicBlock<'llvm> {
        let block = unsafe {
            LLVMAppendBasicBlockInContext(
                self.ctx,
                fn_value.value_ref(),
                b"block\0".as_ptr().cast(),
            )
        };
        assert!(!block.is_null());

        BasicBlock(block, PhantomData)
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.ctx);
        }
    }
}

// ===========
//   Builder
// ===========

/// Wrapper for a LLVM IR Builder.
pub struct Builder<'llvm> {
    builder: LLVMBuilderRef,
    _ctx: PhantomData<&'llvm ()>,
}

impl<'llvm> Builder<'llvm> {
    /// Create a new LLVM IR Builder with the `module`s context.
    ///
    /// # Panics
    ///
    /// Panics if creating the IR Builder fails.
    pub fn with_ctx(module: &'llvm Module) -> Builder<'llvm> {
        let builder = unsafe { LLVMCreateBuilderInContext(module.ctx) };
        assert!(!builder.is_null());

        Builder {
            builder,
            _ctx: PhantomData,
        }
    }

    /// Position the IR Builder at the end of the given Basic Block.
    pub fn pos_at_end(&self, bb: BasicBlock<'llvm>) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, bb.0);
        }
    }

    /// Emit a [fadd](https://llvm.org/docs/LangRef.html#fadd-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn fadd(&self, lhs: Value<'llvm>, rhs: Value<'llvm>) -> Value<'llvm> {
        debug_assert!(lhs.is_f64(), "fadd: Expected f64 as lhs operand!");
        debug_assert!(rhs.is_f64(), "fadd: Expected f64 as rhs operand!");

        let value_ref = unsafe {
            LLVMBuildFAdd(
                self.builder,
                lhs.value_ref(),
                rhs.value_ref(),
                b"fadd\0".as_ptr().cast(),
            )
        };
        Value::new(value_ref)
    }

    /// Emit a [fsub](https://llvm.org/docs/LangRef.html#fsub-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn fsub(&self, lhs: Value<'llvm>, rhs: Value<'llvm>) -> Value<'llvm> {
        debug_assert!(lhs.is_f64(), "fsub: Expected f64 as lhs operand!");
        debug_assert!(rhs.is_f64(), "fsub: Expected f64 as rhs operand!");

        let value_ref = unsafe {
            LLVMBuildFSub(
                self.builder,
                lhs.value_ref(),
                rhs.value_ref(),
                b"fsub\0".as_ptr().cast(),
            )
        };
        Value::new(value_ref)
    }

    /// Emit a [fmul](https://llvm.org/docs/LangRef.html#fmul-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn fmul(&self, lhs: Value<'llvm>, rhs: Value<'llvm>) -> Value<'llvm> {
        debug_assert!(lhs.is_f64(), "fmul: Expected f64 as lhs operand!");
        debug_assert!(rhs.is_f64(), "fmul: Expected f64 as rhs operand!");

        let value_ref = unsafe {
            LLVMBuildFMul(
                self.builder,
                lhs.value_ref(),
                rhs.value_ref(),
                b"fmul\0".as_ptr().cast(),
            )
        };
        Value::new(value_ref)
    }

    /// Emit a [fcmult](https://llvm.org/docs/LangRef.html#fcmp-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn fcmpult(&self, lhs: Value<'llvm>, rhs: Value<'llvm>) -> Value<'llvm> {
        debug_assert!(lhs.is_f64(), "fcmplt: Expected f64 as lhs operand!");
        debug_assert!(rhs.is_f64(), "fcmplt: Expected f64 as rhs operand!");

        let value_ref = unsafe {
            LLVMBuildFCmp(
                self.builder,
                LLVMRealPredicate::LLVMRealULT,
                lhs.value_ref(),
                rhs.value_ref(),
                b"fcmplt\0".as_ptr().cast(),
            )
        };
        Value::new(value_ref)
    }

    /// Emit a [uitofp](https://llvm.org/docs/LangRef.html#uitofp-to-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn uitofp(&self, val: Value<'llvm>, dest_type: Type<'llvm>) -> Value<'llvm> {
        debug_assert!(val.is_int(), "uitofp: Expected integer operand!");

        let value_ref = unsafe {
            LLVMBuildUIToFP(
                self.builder,
                val.value_ref(),
                dest_type.0,
                b"uitofp\0".as_ptr().cast(),
            )
        };
        Value::new(value_ref)
    }

    /// Emit a [call](https://llvm.org/docs/LangRef.html#call-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn call(&self, fn_value: FnValue<'llvm>, args: &mut [Value<'llvm>]) -> Value<'llvm> {
        let value_ref = unsafe {
            LLVMBuildCall2(
                self.builder,
                fn_value.ret_type(),
                fn_value,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                b"call\0".as_ptr().cast(),
            )
        };
        Value::new(value_ref)
    }

    /// Emit a [ret](https://llvm.org/docs/LangRef.html#ret-instruction) instruction.
    ///
    /// # Panics
    ///
    /// Panics if LLVM API returns a `null` pointer.
    pub fn ret(&self, ret: Value<'llvm>) {
        let ret = unsafe { LLVMBuildRet(self.builder, ret.value_ref()) };
        assert!(!ret.is_null());
    }
}

impl Drop for Builder<'_> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.builder) }
    }
}

// ==============
//   BasicBlock
// ==============

/// Wrapper for a LLVM Basic Block.
#[derive(Copy, Clone)]
pub struct BasicBlock<'llvm>(LLVMBasicBlockRef, PhantomData<&'llvm ()>);

// ========
//   Type
// ========

/// Wrapper for a LLVM Type Reference.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Type<'llvm>(LLVMTypeRef, PhantomData<&'llvm ()>);

impl<'llvm> Type<'llvm> {
    fn new(type_ref: LLVMTypeRef) -> Self {
        assert!(!type_ref.is_null());
        Type(type_ref, PhantomData)
    }

    fn kind(&self) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(self.0) }
    }

    /// Dump the LLVM Type to stdout.
    pub fn dump(&self) {
        unsafe { LLVMDumpType(self.0) };
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

        let value_ref = unsafe { LLVMConstReal(self.0, n) };
        Value::new(value_ref)
    }
}

// =========
//   Value
// =========

/// Wrapper for a LLVM Value Reference.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Value<'llvm>(LLVMValueRef, PhantomData<&'llvm ()>);

impl<'llvm> Value<'llvm> {
    fn new(value_ref: LLVMValueRef) -> Self {
        assert!(!value_ref.is_null());
        Value(value_ref, PhantomData)
    }

    #[inline]
    fn value_ref(&self) -> LLVMValueRef {
        self.0
    }

    fn kind(&self) -> LLVMValueKind {
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
    fn new(value_ref: LLVMValueRef) -> Self {
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
