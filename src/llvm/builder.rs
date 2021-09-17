use llvm_sys::{
    core::{
        LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFMul, LLVMBuildFSub, LLVMBuildRet, LLVMBuildUIToFP,
        LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMPositionBuilderAtEnd,
    },
    prelude::{LLVMBuilderRef, LLVMValueRef},
    LLVMRealPredicate,
};

use std::marker::PhantomData;

use super::{BasicBlock, FnValue, Module, Type, Value};

// Definition of LLVM C API functions using our `repr(transparent)` types.
extern "C" {
    fn LLVMBuildCall2(
        arg1: LLVMBuilderRef,
        arg2: Type<'_>,
        Fn: FnValue<'_>,
        Args: *mut Value<'_>,
        NumArgs: ::libc::c_uint,
        Name: *const ::libc::c_char,
    ) -> LLVMValueRef;
}

/// Wrapper for a LLVM IR Builder.
pub struct IRBuilder<'llvm> {
    builder: LLVMBuilderRef,
    _ctx: PhantomData<&'llvm ()>,
}

impl<'llvm> IRBuilder<'llvm> {
    /// Create a new LLVM IR Builder with the `module`s context.
    ///
    /// # Panics
    ///
    /// Panics if creating the IR Builder fails.
    pub fn with_ctx(module: &'llvm Module) -> IRBuilder<'llvm> {
        let builder = unsafe { LLVMCreateBuilderInContext(module.ctx()) };
        assert!(!builder.is_null());

        IRBuilder {
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
                dest_type.type_ref(),
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

impl Drop for IRBuilder<'_> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.builder) }
    }
}
