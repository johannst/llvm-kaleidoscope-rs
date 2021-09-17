use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMContextCreate, LLVMContextDispose,
        LLVMDisposeModule, LLVMDoubleTypeInContext, LLVMDumpModule, LLVMGetNamedFunction,
        LLVMModuleCreateWithNameInContext,
    },
    prelude::{LLVMBool, LLVMContextRef, LLVMModuleRef, LLVMTypeRef},
    LLVMTypeKind,
};

use std::convert::TryFrom;
use std::marker::PhantomData;

use super::{BasicBlock, FnValue, Type};
use crate::SmallCStr;

// Definition of LLVM C API functions using our `repr(transparent)` types.
extern "C" {
    fn LLVMFunctionType(
        ReturnType: Type<'_>,
        ParamTypes: *mut Type<'_>,
        ParamCount: ::libc::c_uint,
        IsVarArg: LLVMBool,
    ) -> LLVMTypeRef;
}

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

    /// Get the raw LLVM context reference.
    #[inline]
    pub(super) fn ctx(&self) -> LLVMContextRef {
        self.ctx
    }

    /// Get the raw LLVM module reference.
    #[inline]
    pub(super) fn module(&self) -> LLVMModuleRef {
        self.module
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

        let value_ref = unsafe { LLVMAddFunction(self.module, name.as_ptr(), fn_type.type_ref()) };
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
