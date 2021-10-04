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
    fn_protos: &'a mut HashMap<String, PrototypeAST>,
}

impl<'llvm, 'a> Codegen<'llvm, 'a> {
    /// Compile either a [`PrototypeAST`] or a [`FunctionAST`] into the LLVM `module`.
    pub fn compile(
        module: &'llvm Module,
        fn_protos: &mut HashMap<String, PrototypeAST>,
        compilee: Either<&PrototypeAST, &FunctionAST>,
    ) -> CodegenResult<FnValue<'llvm>> {
        let mut cg = Codegen {
            module,
            builder: &IRBuilder::with_ctx(module),
            fpm: &FunctionPassManager::with_ctx(module),
            fn_protos,
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
            ExprAST::Call(callee, args) => match self.get_function(callee) {
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
            ExprAST::If { cond, then, else_ } => {
                // For 'if' expressions we are building the following CFG.
                //
                //         ; cond
                //         br
                //          |
                //    +-----+------+
                //    v            v
                //  ; then       ; else
                //    |            |
                //    +-----+------+
                //          v
                //        ; merge
                //        phi then, else
                //        ret phi

                let cond_v = {
                    // Codgen 'cond' expression.
                    let v = self.codegen_expr(cond, named_values)?;
                    // Convert condition to bool.
                    self.builder
                        .fcmpone(v, self.module.type_f64().const_f64(0f64))
                };

                // Get the function we are currently inserting into.
                let the_function = self.builder.get_insert_block().get_parent();

                // Create basic blocks for the 'then' / 'else' expressions as well as the return
                // instruction ('merge').
                //
                // Append the 'then' basic block to the function, don't insert the 'else' and
                // 'merge' basic blocks yet.
                let then_bb = self.module.append_basic_block(the_function);
                let else_bb = self.module.create_basic_block();
                let merge_bb = self.module.create_basic_block();

                // Create a conditional branch based on the result of the 'cond' expression.
                self.builder.cond_br(cond_v, then_bb, else_bb);

                // Move to 'then' basic block and codgen the 'then' expression.
                self.builder.pos_at_end(then_bb);
                let then_v = self.codegen_expr(then, named_values)?;
                // Create unconditional branch to 'merge' block.
                self.builder.br(merge_bb);
                // Update reference to current basic block (in case the 'then' expression added new
                // basic blocks).
                let then_bb = self.builder.get_insert_block();

                // Now append the 'else' basic block to the function.
                the_function.append_basic_block(else_bb);
                // Move to 'else' basic block and codgen the 'else' expression.
                self.builder.pos_at_end(else_bb);
                let else_v = self.codegen_expr(else_, named_values)?;
                // Create unconditional branch to 'merge' block.
                self.builder.br(merge_bb);
                // Update reference to current basic block (in case the 'else' expression added new
                // basic blocks).
                let else_bb = self.builder.get_insert_block();

                // Now append the 'merge' basic block to the function.
                the_function.append_basic_block(merge_bb);
                // Move to 'merge' basic block.
                self.builder.pos_at_end(merge_bb);
                // Codegen the phi node returning the appropriate value depending on the branch
                // condition.
                let phi = self.builder.phi(
                    self.module.type_f64(),
                    &[(then_v, then_bb), (else_v, else_bb)],
                );

                Ok(phi)
            }
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
        &mut self,
        FunctionAST(proto, body): &FunctionAST,
        named_values: &mut HashMap<&'llvm str, Value<'llvm>>,
    ) -> CodegenResult<FnValue<'llvm>> {
        // Insert the function prototype into the `fn_protos` map to keep track for re-generating
        // declarations in other modules.
        self.fn_protos.insert(proto.0.clone(), proto.clone());

        let the_function = self.get_function(&proto.0)
            .expect("If proto not already generated, get_function will do for us since we updated fn_protos before-hand!");

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

    /// Lookup function with `name` in the LLVM module and return the corresponding value reference.
    /// If the function is not available in the module, check if the prototype is known and codegen
    /// it.
    /// Return [`None`] if the prototype is not known.
    fn get_function(&self, name: &str) -> Option<FnValue<'llvm>> {
        let callee = match self.module.get_fn(name) {
            Some(callee) => callee,
            None => {
                let proto = self.fn_protos.get(name)?;
                self.codegen_prototype(proto)
            }
        };

        Some(callee)
    }
}
