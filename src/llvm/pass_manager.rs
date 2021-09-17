use llvm_sys::{
    core::{
        LLVMCreateFunctionPassManagerForModule, LLVMDisposePassManager,
        LLVMInitializeFunctionPassManager, LLVMRunFunctionPassManager,
    },
    prelude::LLVMPassManagerRef,
    transforms::{
        instcombine::LLVMAddInstructionCombiningPass,
        scalar::{LLVMAddCFGSimplificationPass, LLVMAddNewGVNPass, LLVMAddReassociatePass},
    },
};

use std::marker::PhantomData;

use super::{FnValue, Module};

/// Wrapper for a LLVM Function PassManager (legacy).
pub struct FunctionPassManager<'llvm> {
    fpm: LLVMPassManagerRef,
    _ctx: PhantomData<&'llvm ()>,
}

impl<'llvm> FunctionPassManager<'llvm> {
    /// Create a new Function PassManager with the following optimization passes
    /// - InstructionCombiningPass
    /// - ReassociatePass
    /// - NewGVNPass
    /// - CFGSimplificationPass
    ///
    /// The list of selected optimization passes is taken from the tutorial chapter [LLVM
    /// Optimization Passes](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl04.html#id3).
    pub fn with_ctx(module: &'llvm Module) -> FunctionPassManager<'llvm> {
        let fpm = unsafe {
            // Borrows module reference.
            LLVMCreateFunctionPassManagerForModule(module.module())
        };
        assert!(!fpm.is_null());

        unsafe {
            // Do simple "peephole" optimizations and bit-twiddling optzns.
            LLVMAddInstructionCombiningPass(fpm);
            // Reassociate expressions.
            LLVMAddReassociatePass(fpm);
            // Eliminate Common SubExpressions.
            LLVMAddNewGVNPass(fpm);
            // Simplify the control flow graph (deleting unreachable blocks, etc).
            LLVMAddCFGSimplificationPass(fpm);

            let fail = LLVMInitializeFunctionPassManager(fpm);
            assert_eq!(fail, 0);
        }

        FunctionPassManager {
            fpm,
            _ctx: PhantomData,
        }
    }

    /// Run the optimization passes registered with the Function PassManager on the function
    /// referenced by `fn_value`.
    pub fn run(&'llvm self, fn_value: FnValue<'llvm>) {
        unsafe {
            // Returns 1 if any of the passes modified the function, false otherwise.
            LLVMRunFunctionPassManager(self.fpm, fn_value.value_ref());
        }
    }
}

impl Drop for FunctionPassManager<'_> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposePassManager(self.fpm);
        }
    }
}
