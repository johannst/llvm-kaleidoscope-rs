//! Safe wrapper around the LLVM C API.
//!
//! References returned from the LLVM API are tied to the `'llvm` lifetime which is bound to the
//! context where the objects are created in.
//! We do not offer wrappers to remove or delete any objects in the context and therefore all the
//! references will be valid for the liftime of the context.
//!
//! For the scope of this tutorial we mainly use assertions to validate the results from the LLVM
//! API calls.

use llvm_sys::{core::LLVMShutdown, prelude::LLVMBasicBlockRef};

use std::marker::PhantomData;

mod builder;
mod module;
mod pass_manager;
mod type_;
mod value;

pub use builder::IRBuilder;
pub use module::Module;
pub use pass_manager::FunctionPassManager;
pub use type_::Type;
pub use value::{FnValue, Value};

/// Wrapper for a LLVM Basic Block.
#[derive(Copy, Clone)]
pub struct BasicBlock<'llvm>(LLVMBasicBlockRef, PhantomData<&'llvm ()>);

/// Deallocate and destroy all "ManagedStatic" variables.
pub fn shutdown() {
    unsafe {
        LLVMShutdown();
    };
}
