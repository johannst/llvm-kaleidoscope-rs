use std::collections::HashMap;

use crate::llvm::{FnValue, FunctionPassManager, IRBuilder, Module, Value};
use crate::parser::{ExprAST, FunctionAST, PrototypeAST};
use crate::Either;

type CodegenResult<T> = Result<T, String>;

/// Code generator from kaleidoscope AST to LLVM IR.
pub struct Codegen<'llvm, 'a> {
    module: &'llvm Module,
    builder: &'a IRBuilder<'llvm>,
    fpm: &'a FunctionPassManager<'llvm>,
}

impl<'llvm, 'a> Codegen<'llvm, 'a> {
    /// Compile either a [`PrototypeAST`] or a [`FunctionAST`] into the LLVM `module`.
    pub fn compile(
        module: &'llvm Module,
        compilee: Either<&PrototypeAST, &FunctionAST>,
    ) -> CodegenResult<FnValue<'llvm>> {
        let cg = Codegen {
            module,
            builder: &IRBuilder::with_ctx(module),
            fpm: &FunctionPassManager::with_ctx(module),
        };
        let mut variables = HashMap::new();

        match compilee {
            Either::A(proto) => Ok(cg.codegen_prototype(proto)),
            Either::B(func) => cg.codegen_function(func, &mut variables),
        }
    }

    fn codegen_expr(
        &self,
        expr: &ExprAST,
        named_values: &mut HashMap<&'llvm str, Value<'llvm>>,
    ) -> CodegenResult<Value<'llvm>> {
        match expr {
            ExprAST::Number(num) => Ok(self.module.type_f64().const_f64(*num)),
            ExprAST::Variable(name) => match named_values.get(name.as_str()) {
                Some(value) => Ok(*value),
                None => Err("Unknown variable name".into()),
            },
            ExprAST::Binary(binop, lhs, rhs) => {
                let l = self.codegen_expr(lhs, named_values)?;
                let r = self.codegen_expr(rhs, named_values)?;

                match binop {
                    '+' => Ok(self.builder.fadd(l, r)),
                    '-' => Ok(self.builder.fsub(l, r)),
                    '*' => Ok(self.builder.fmul(l, r)),
                    '<' => {
                        let res = self.builder.fcmpult(l, r);
                        // Turn bool into f64.
                        Ok(self.builder.uitofp(res, self.module.type_f64()))
                    }
                    _ => Err("invalid binary operator".into()),
                }
            }
            ExprAST::Call(callee, args) => match self.module.get_fn(callee) {
                Some(callee) => {
                    if callee.args() != args.len() {
                        return Err("Incorrect # arguments passed".into());
                    }

                    // Generate code for function argument expressions.
                    let mut args: Vec<Value<'_>> = args
                        .iter()
                        .map(|arg| self.codegen_expr(arg, named_values))
                        .collect::<CodegenResult<_>>()?;

                    Ok(self.builder.call(callee, &mut args))
                }
                None => Err("Unknown function referenced".into()),
            },
        }
    }

    fn codegen_prototype(&self, PrototypeAST(name, args): &PrototypeAST) -> FnValue<'llvm> {
        let type_f64 = self.module.type_f64();

        let mut doubles = Vec::new();
        doubles.resize(args.len(), type_f64);

        // Build the function type: fn(f64, f64, ..) -> f64
        let ft = self.module.type_fn(&mut doubles, type_f64);

        // Create the function declaration.
        let f = self.module.add_fn(name, ft);

        // Set the names of the function arguments.
        for idx in 0..f.args() {
            f.arg(idx).set_name(&args[idx]);
        }

        f
    }

    fn codegen_function(
        &self,
        FunctionAST(proto, body): &FunctionAST,
        named_values: &mut HashMap<&'llvm str, Value<'llvm>>,
    ) -> CodegenResult<FnValue<'llvm>> {
        let the_function = match self.module.get_fn(&proto.0) {
            Some(f) => f,
            None => self.codegen_prototype(proto),
        };

        if the_function.basic_blocks() > 0 {
            return Err("Function cannot be redefined.".into());
        }

        // Create entry basic block to insert code.
        let bb = self.module.append_basic_block(the_function);
        self.builder.pos_at_end(bb);

        // New scope, clear the map with the function args.
        named_values.clear();

        // Update the map with the current functions args.
        for idx in 0..the_function.args() {
            let arg = the_function.arg(idx);
            named_values.insert(arg.get_name(), arg);
        }

        // Codegen function body.
        if let Ok(ret) = self.codegen_expr(body, named_values) {
            self.builder.ret(ret);
            assert!(the_function.verify());

            // Run the optimization passes on the function.
            self.fpm.run(the_function);

            Ok(the_function)
        } else {
            todo!("Failed to codegen function body, erase from module!");
        }
    }
}
