use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum ExprAST {
    /// Number - Expression class for numeric literals like "1.0".
    Number(f64),

    /// Variable - Expression class for referencing a variable, like "a".
    Variable(String),

    /// Binary - Expression class for a binary operator.
    Binary(char, Box<ExprAST>, Box<ExprAST>),

    /// Call - Expression class for function calls.
    Call(String, Vec<ExprAST>),

    /// If - Expression class for if/then/else.
    If {
        cond: Box<ExprAST>,
        then: Box<ExprAST>,
        else_: Box<ExprAST>,
    },

    /// ForExprAST - Expression class for for/in.
    For {
        var: String,
        start: Box<ExprAST>,
        end: Box<ExprAST>,
        step: Option<Box<ExprAST>>,
        body: Box<ExprAST>,
    },
}

/// PrototypeAST - This class represents the "prototype" for a function,
/// which captures its name, and its argument names (thus implicitly the number
/// of arguments the function takes).
#[derive(Debug, PartialEq, Clone)]
pub struct PrototypeAST(pub String, pub Vec<String>);

/// FunctionAST - This class represents a function definition itself.
#[derive(Debug, PartialEq)]
pub struct FunctionAST(pub PrototypeAST, pub ExprAST);

/// Parse result with String as Error type (to be compliant with tutorial).
type ParseResult<T> = Result<T, String>;

/// Parser for the `kaleidoscope` language.
pub struct Parser<I>
where
    I: Iterator<Item = char>,
{
    lexer: Lexer<I>,
    cur_tok: Option<Token>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(lexer: Lexer<I>) -> Self {
        Parser {
            lexer,
            cur_tok: None,
        }
    }

    // -----------------------
    //   Simple Token Buffer
    // -----------------------

    /// Implement the global variable `int CurTok;` from the tutorial.
    ///
    /// # Panics
    /// Panics if the parser doesn't have a current token.
    pub fn cur_tok(&self) -> &Token {
        self.cur_tok.as_ref().expect("Parser: Expected cur_token!")
    }

    /// Advance the `cur_tok` by getting the next token from the lexer.
    ///
    /// Implement the fucntion `int getNextToken();` from the tutorial.
    pub fn get_next_token(&mut self) {
        self.cur_tok = Some(self.lexer.gettok());
    }

    // ----------------------------
    //   Basic Expression Parsing
    // ----------------------------

    /// numberexpr ::= number
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseNumberExpr();` from the tutorial.
    fn parse_num_expr(&mut self) -> ParseResult<ExprAST> {
        match *self.cur_tok() {
            Token::Number(num) => {
                // Consume the number token.
                self.get_next_token();
                Ok(ExprAST::Number(num))
            }
            _ => unreachable!(),
        }
    }

    /// parenexpr ::= '(' expression ')'
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseParenExpr();` from the tutorial.
    fn parse_paren_expr(&mut self) -> ParseResult<ExprAST> {
        // Eat '(' token.
        assert_eq!(*self.cur_tok(), Token::Char('('));
        self.get_next_token();

        let v = self.parse_expression()?;

        if *self.cur_tok() == Token::Char(')') {
            // Eat ')' token.
            self.get_next_token();
            Ok(v)
        } else {
            Err("expected ')'".into())
        }
    }

    /// identifierexpr
    ///   ::= identifier
    ///   ::= identifier '(' expression* ')'
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseIdentifierExpr();` from the tutorial.
    fn parse_identifier_expr(&mut self) -> ParseResult<ExprAST> {
        let id_name = match self.cur_tok.take() {
            Some(Token::Identifier(id)) => {
                // Consume identifier.
                self.get_next_token();
                id
            }
            _ => unreachable!(),
        };

        if *self.cur_tok() != Token::Char('(') {
            // Simple variable reference.
            Ok(ExprAST::Variable(id_name))
        } else {
            // Call.

            // Eat '(' token.
            self.get_next_token();

            let mut args: Vec<ExprAST> = Vec::new();

            // If there are arguments collect them.
            if *self.cur_tok() != Token::Char(')') {
                loop {
                    let arg = self.parse_expression()?;
                    args.push(arg);

                    if *self.cur_tok() == Token::Char(')') {
                        break;
                    }

                    if *self.cur_tok() != Token::Char(',') {
                        return Err("Expected ')' or ',' in argument list".into());
                    }

                    self.get_next_token();
                }
            }

            assert_eq!(*self.cur_tok(), Token::Char(')'));
            // Eat ')' token.
            self.get_next_token();

            Ok(ExprAST::Call(id_name, args))
        }
    }

    /// ifexpr ::= 'if' expression 'then' expression 'else' expression
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseIfExpr();` from the tutorial.
    fn parse_if_expr(&mut self) -> ParseResult<ExprAST> {
        // Consume 'if' token.
        assert_eq!(*self.cur_tok(), Token::If);
        self.get_next_token();

        let cond = self.parse_expression()?;

        if *dbg!(self.cur_tok()) != Token::Then {
            return Err("Expected 'then'".into());
        }
        // Consume 'then' token.
        self.get_next_token();

        let then = self.parse_expression()?;

        if *self.cur_tok() != Token::Else {
            return Err("Expected 'else'".into());
        }
        // Consume 'else' token.
        self.get_next_token();

        let else_ = self.parse_expression()?;

        Ok(ExprAST::If {
            cond: Box::new(cond),
            then: Box::new(then),
            else_: Box::new(else_),
        })
    }

    /// forexpr ::= 'for' identifier '=' expr ',' expr (',' expr)? 'in' expression
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseForExpr();` from the tutorial.
    fn parse_for_expr(&mut self) -> ParseResult<ExprAST> {
        // Consume the 'for' token.
        assert_eq!(*self.cur_tok(), Token::For);
        self.get_next_token();

        let var = match self
            .parse_identifier_expr()
            .map_err(|_| String::from("expected identifier after 'for'"))?
        {
            ExprAST::Variable(var) => var,
            _ => unreachable!(),
        };

        // Consume the '=' token.
        if *self.cur_tok() != Token::Char('=') {
            return Err("expected '=' after for".into());
        }
        self.get_next_token();

        let start = self.parse_expression()?;

        // Consume the ',' token.
        if *self.cur_tok() != Token::Char(',') {
            return Err("expected ',' after for start value".into());
        }
        self.get_next_token();

        let end = self.parse_expression()?;

        let step = if *self.cur_tok() == Token::Char(',') {
            // Consume the ',' token.
            self.get_next_token();

            Some(self.parse_expression()?)
        } else {
            None
        };

        // Consume the 'in' token.
        if *self.cur_tok() != Token::In {
            return Err("expected 'in' after for".into());
        }
        self.get_next_token();

        let body = self.parse_expression()?;

        Ok(ExprAST::For {
            var,
            start: Box::new(start),
            end: Box::new(end),
            step: step.map(|s| Box::new(s)),
            body: Box::new(body),
        })
    }

    /// primary
    ///   ::= identifierexpr
    ///   ::= numberexpr
    ///   ::= parenexpr
    ///
    /// Implement `std::unique_ptr<ExprAST> ParsePrimary();` from the tutorial.
    fn parse_primary(&mut self) -> ParseResult<ExprAST> {
        match *self.cur_tok() {
            Token::Identifier(_) => self.parse_identifier_expr(),
            Token::Number(_) => self.parse_num_expr(),
            Token::Char('(') => self.parse_paren_expr(),
            Token::If => self.parse_if_expr(),
            Token::For => self.parse_for_expr(),
            _ => Err("unknown token when expecting an expression".into()),
        }
    }

    // -----------------------------
    //   Binary Expression Parsing
    // -----------------------------

    /// /// expression
    ///   ::= primary binoprhs
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseExpression();` from the tutorial.
    fn parse_expression(&mut self) -> ParseResult<ExprAST> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    /// binoprhs
    ///   ::= ('+' primary)*
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseBinOpRHS(int ExprPrec, std::unique_ptr<ExprAST> LHS);` from the tutorial.
    fn parse_bin_op_rhs(&mut self, expr_prec: isize, mut lhs: ExprAST) -> ParseResult<ExprAST> {
        loop {
            let tok_prec = get_tok_precedence(self.cur_tok());

            // Not a binary operator or precedence is too small.
            if tok_prec < expr_prec {
                return Ok(lhs);
            }

            let binop = match self.cur_tok.take() {
                Some(Token::Char(c)) => {
                    // Eat binary operator.
                    self.get_next_token();
                    c
                }
                _ => unreachable!(),
            };

            // lhs BINOP1 rhs BINOP2 remrhs
            //     ^^^^^^     ^^^^^^
            //     tok_prec   next_prec
            //
            // In case BINOP1 has higher precedence, we are done here and can build a 'Binary' AST
            // node between 'lhs' and 'rhs'.
            //
            // In case BINOP2 has higher precedence, we take 'rhs' as 'lhs' and recurse into the
            // 'remrhs' expression first.

            // Parse primary expression after binary operator.
            let mut rhs = self.parse_primary()?;

            let next_prec = get_tok_precedence(self.cur_tok());
            if tok_prec < next_prec {
                // BINOP2 has higher precedence thatn BINOP1, recurse into 'remhs'.
                rhs = self.parse_bin_op_rhs(tok_prec + 1, rhs)?
            }

            lhs = ExprAST::Binary(binop, Box::new(lhs), Box::new(rhs));
        }
    }

    // --------------------
    //   Parsing the Rest
    // --------------------

    /// prototype
    ///   ::= id '(' id* ')'
    ///
    /// Implement `std::unique_ptr<PrototypeAST> ParsePrototype();` from the tutorial.
    fn parse_prototype(&mut self) -> ParseResult<PrototypeAST> {
        let id_name = match self.cur_tok.take() {
            Some(Token::Identifier(id)) => {
                // Consume the identifier.
                self.get_next_token();
                id
            }
            other => {
                // Plug back current token.
                self.cur_tok = other;
                return Err("Expected function name in prototype".into());
            }
        };

        if *self.cur_tok() != Token::Char('(') {
            return Err("Expected '(' in prototype".into());
        }

        let mut args: Vec<String> = Vec::new();
        loop {
            self.get_next_token();

            match self.cur_tok.take() {
                Some(Token::Identifier(arg)) => args.push(arg),
                Some(Token::Char(',')) => {}
                other => {
                    self.cur_tok = other;
                    break;
                }
            }
        }

        if *self.cur_tok() != Token::Char(')') {
            return Err("Expected ')' in prototype".into());
        }

        // Consume ')'.
        self.get_next_token();

        Ok(PrototypeAST(id_name, args))
    }

    /// definition ::= 'def' prototype expression
    ///
    /// Implement `std::unique_ptr<FunctionAST> ParseDefinition();` from the tutorial.
    pub fn parse_definition(&mut self) -> ParseResult<FunctionAST> {
        // Consume 'def' token.
        assert_eq!(*self.cur_tok(), Token::Def);
        self.get_next_token();

        let proto = self.parse_prototype()?;
        let expr = self.parse_expression()?;

        Ok(FunctionAST(proto, expr))
    }

    /// external ::= 'extern' prototype
    ///
    /// Implement `std::unique_ptr<PrototypeAST> ParseExtern();` from the tutorial.
    pub fn parse_extern(&mut self) -> ParseResult<PrototypeAST> {
        // Consume 'extern' token.
        assert_eq!(*self.cur_tok(), Token::Extern);
        self.get_next_token();

        self.parse_prototype()
    }

    /// toplevelexpr ::= expression
    ///
    /// Implement `std::unique_ptr<FunctionAST> ParseTopLevelExpr();` from the tutorial.
    pub fn parse_top_level_expr(&mut self) -> ParseResult<FunctionAST> {
        let e = self.parse_expression()?;
        let proto = PrototypeAST("__anon_expr".into(), Vec::new());
        Ok(FunctionAST(proto, e))
    }
}

/// Get the binary operator precedence.
///
/// Implement `int GetTokPrecedence();` from the tutorial.
fn get_tok_precedence(tok: &Token) -> isize {
    match tok {
        Token::Char('<') => 10,
        Token::Char('+') => 20,
        Token::Char('-') => 20,
        Token::Char('*') => 40,
        _ => -1,
    }
}

#[cfg(test)]
mod test {
    use super::{ExprAST, FunctionAST, Parser, PrototypeAST};
    use crate::lexer::Lexer;

    fn parser(input: &str) -> Parser<std::str::Chars> {
        let l = Lexer::new(input.chars());
        let mut p = Parser::new(l);

        // Drop initial coin, initialize cur_tok.
        p.get_next_token();

        p
    }

    #[test]
    fn parse_number() {
        let mut p = parser("13.37");

        assert_eq!(p.parse_num_expr(), Ok(ExprAST::Number(13.37f64)));
    }

    #[test]
    fn parse_variable() {
        let mut p = parser("foop");

        assert_eq!(
            p.parse_identifier_expr(),
            Ok(ExprAST::Variable("foop".into()))
        );
    }

    #[test]
    fn parse_if() {
        let mut p = parser("if 1 then 2 else 3");

        let cond = Box::new(ExprAST::Number(1f64));
        let then = Box::new(ExprAST::Number(2f64));
        let else_ = Box::new(ExprAST::Number(3f64));

        assert_eq!(p.parse_if_expr(), Ok(ExprAST::If { cond, then, else_ }));

        let mut p = parser("if foo() then bar(2) else baz(3)");

        let cond = Box::new(ExprAST::Call("foo".into(), vec![]));
        let then = Box::new(ExprAST::Call("bar".into(), vec![ExprAST::Number(2f64)]));
        let else_ = Box::new(ExprAST::Call("baz".into(), vec![ExprAST::Number(3f64)]));

        assert_eq!(p.parse_if_expr(), Ok(ExprAST::If { cond, then, else_ }));
    }

    #[test]
    fn parse_for() {
        let mut p = parser("for i = 1, 2, 3 in 4");

        let var = String::from("i");
        let start = Box::new(ExprAST::Number(1f64));
        let end = Box::new(ExprAST::Number(2f64));
        let step = Some(Box::new(ExprAST::Number(3f64)));
        let body = Box::new(ExprAST::Number(4f64));

        assert_eq!(
            p.parse_for_expr(),
            Ok(ExprAST::For {
                var,
                start,
                end,
                step,
                body
            })
        );
    }

    #[test]
    fn parse_for_no_step() {
        let mut p = parser("for i = 1, 2 in 4");

        let var = String::from("i");
        let start = Box::new(ExprAST::Number(1f64));
        let end = Box::new(ExprAST::Number(2f64));
        let step = None;
        let body = Box::new(ExprAST::Number(4f64));

        assert_eq!(
            p.parse_for_expr(),
            Ok(ExprAST::For {
                var,
                start,
                end,
                step,
                body
            })
        );
    }

    #[test]
    fn parse_primary() {
        let mut p = parser("1337 foop \n bla(123) \n if a then b else c \n for x=1,2 in 3");

        assert_eq!(p.parse_primary(), Ok(ExprAST::Number(1337f64)));

        assert_eq!(p.parse_primary(), Ok(ExprAST::Variable("foop".into())));

        assert_eq!(
            p.parse_primary(),
            Ok(ExprAST::Call("bla".into(), vec![ExprAST::Number(123f64)]))
        );

        assert_eq!(
            p.parse_primary(),
            Ok(ExprAST::If {
                cond: Box::new(ExprAST::Variable("a".into())),
                then: Box::new(ExprAST::Variable("b".into())),
                else_: Box::new(ExprAST::Variable("c".into())),
            })
        );

        assert_eq!(
            p.parse_primary(),
            Ok(ExprAST::For {
                var: String::from("x"),
                start: Box::new(ExprAST::Number(1f64)),
                end: Box::new(ExprAST::Number(2f64)),
                step: None,
                body: Box::new(ExprAST::Number(3f64)),
            })
        );
    }

    #[test]
    fn parse_binary_op() {
        // Operator before RHS has higher precedence, expected AST
        //
        //       -
        //      / \
        //     +     c
        //    / \
        //   a   b
        let mut p = parser("a + b - c");

        let binexpr_ab = ExprAST::Binary(
            '+',
            Box::new(ExprAST::Variable("a".into())),
            Box::new(ExprAST::Variable("b".into())),
        );

        let binexpr_abc = ExprAST::Binary(
            '-',
            Box::new(binexpr_ab),
            Box::new(ExprAST::Variable("c".into())),
        );

        assert_eq!(p.parse_expression(), Ok(binexpr_abc));
    }

    #[test]
    fn parse_binary_op2() {
        // Operator after RHS has higher precedence, expected AST
        //
        //       +
        //      / \
        //     a   *
        //        / \
        //       b   c
        let mut p = parser("a + b * c");

        let binexpr_bc = ExprAST::Binary(
            '*',
            Box::new(ExprAST::Variable("b".into())),
            Box::new(ExprAST::Variable("c".into())),
        );

        let binexpr_abc = ExprAST::Binary(
            '+',
            Box::new(ExprAST::Variable("a".into())),
            Box::new(binexpr_bc),
        );

        assert_eq!(p.parse_expression(), Ok(binexpr_abc));
    }

    #[test]
    fn parse_prototype() {
        let mut p = parser("foo(a,b)");

        let proto = PrototypeAST("foo".into(), vec!["a".into(), "b".into()]);

        assert_eq!(p.parse_prototype(), Ok(proto));
    }

    #[test]
    fn parse_definition() {
        let mut p = parser("def bar( arg0 , arg1 ) arg0 + arg1");

        let proto = PrototypeAST("bar".into(), vec!["arg0".into(), "arg1".into()]);

        let body = ExprAST::Binary(
            '+',
            Box::new(ExprAST::Variable("arg0".into())),
            Box::new(ExprAST::Variable("arg1".into())),
        );

        let func = FunctionAST(proto, body);

        assert_eq!(p.parse_definition(), Ok(func));
    }

    #[test]
    fn parse_extern() {
        let mut p = parser("extern baz()");

        let proto = PrototypeAST("baz".into(), vec![]);

        assert_eq!(p.parse_extern(), Ok(proto));
    }
}
