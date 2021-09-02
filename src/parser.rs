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
    Call(String, Vec<Box<ExprAST>>),
}

/// PrototypeAST - This class represents the "prototype" for a function,
/// which captures its name, and its argument names (thus implicitly the number
/// of arguments the function takes).
#[derive(Debug)]
pub struct PrototypeAST(String, Vec<String>);

/// FunctionAST - This class represents a function definition itself.
#[derive(Debug)]
pub struct FunctionAST(Box<PrototypeAST>, Box<ExprAST>);

/// Parse result with String as Error type (to be compliant with tutorial).
type ParseResult<T> = Result<T, String>;

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
    fn parse_num_expr(&mut self) -> ParseResult<Box<ExprAST>> {
        match *self.cur_tok() {
            Token::Number(num) => {
                // Consume the number token.
                self.get_next_token();
                Ok(Box::new(ExprAST::Number(num)))
            }
            _ => unreachable!(),
        }
    }

    /// parenexpr ::= '(' expression ')'
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseParenExpr();` from the tutorial.
    fn parse_paren_expr(&mut self) -> ParseResult<Box<ExprAST>> {
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
    fn parse_identifier_expr(&mut self) -> ParseResult<Box<ExprAST>> {
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
            Ok(Box::new(ExprAST::Variable(id_name)))
        } else {
            // Call.

            // Eat '(' token.
            self.get_next_token();

            let mut args: Vec<Box<ExprAST>> = Vec::new();

            // If there are arguments collect them.
            if *self.cur_tok() != Token::Char(')') {
                loop {
                    let arg = self.parse_expression()?;
                    args.push(arg);

                    if *self.cur_tok() == Token::Char(')') {
                        // Eat ')' token.
                        self.get_next_token();
                        break;
                    }

                    if *self.cur_tok() != Token::Char(',') {
                        return Err("Expected ')' or ',' in argument list".into());
                    }

                    self.get_next_token();
                }
            }

            Ok(Box::new(ExprAST::Call(id_name, args)))
        }
    }

    /// primary
    ///   ::= identifierexpr
    ///   ::= numberexpr
    ///   ::= parenexpr
    ///
    /// Implement `std::unique_ptr<ExprAST> ParsePrimary();` from the tutorial.
    fn parse_primary(&mut self) -> ParseResult<Box<ExprAST>> {
        match *self.cur_tok() {
            Token::Identifier(_) => self.parse_identifier_expr(),
            Token::Number(_) => self.parse_num_expr(),
            Token::Char('(') => self.parse_paren_expr(),
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
    fn parse_expression(&mut self) -> ParseResult<Box<ExprAST>> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    /// binoprhs
    ///   ::= ('+' primary)*
    ///
    /// Implement `std::unique_ptr<ExprAST> ParseBinOpRHS(int ExprPrec, std::unique_ptr<ExprAST> LHS);` from the tutorial.
    fn parse_bin_op_rhs(
        &mut self,
        expr_prec: isize,
        mut lhs: Box<ExprAST>,
    ) -> ParseResult<Box<ExprAST>> {
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

            lhs = Box::new(ExprAST::Binary(binop, lhs, rhs));
        }
    }

    // --------------------
    //   Parsing the Rest
    // --------------------

    /// prototype
    ///   ::= id '(' id* ')'
    ///
    /// Implement `std::unique_ptr<PrototypeAST> ParsePrototype();` from the tutorial.
    fn parse_prototype(&mut self) -> ParseResult<Box<PrototypeAST>> {
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

        Ok(Box::new(PrototypeAST(id_name, args)))
    }

    /// definition ::= 'def' prototype expression
    ///
    /// Implement `std::unique_ptr<FunctionAST> ParseDefinition();` from the tutorial.
    pub fn parse_definition(&mut self) -> ParseResult<Box<FunctionAST>> {
        // Consume 'def' token.
        assert_eq!(*self.cur_tok(), Token::Def);
        self.get_next_token();

        let proto = self.parse_prototype()?;
        let expr = self.parse_expression()?;

        Ok(Box::new(FunctionAST(proto, expr)))
    }

    /// external ::= 'extern' prototype
    ///
    /// Implement `std::unique_ptr<PrototypeAST> ParseExtern();` from the tutorial.
    pub fn parse_extern(&mut self) -> ParseResult<Box<PrototypeAST>> {
        // Consume 'extern' token.
        assert_eq!(*self.cur_tok(), Token::Extern);
        self.get_next_token();

        self.parse_prototype()
    }

    /// toplevelexpr ::= expression
    ///
    /// Implement `std::unique_ptr<FunctionAST> ParseTopLevelExpr();` from the tutorial.
    pub fn parse_top_level_expr(&mut self) -> ParseResult<Box<FunctionAST>> {
        let e = self.parse_expression()?;
        let proto = Box::new(PrototypeAST("".into(), Vec::new()));
        Ok(Box::new(FunctionAST(proto, e)))
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
    use super::{ExprAST, Parser};
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

        assert_eq!(p.parse_num_expr(), Ok(Box::new(ExprAST::Number(13.37f64))));
    }

    #[test]
    fn parse_variable() {
        let mut p = parser("foop");

        assert_eq!(
            p.parse_identifier_expr(),
            Ok(Box::new(ExprAST::Variable("foop".into())))
        );
    }

    #[test]
    fn parse_primary() {
        let mut p = parser("1337 foop \n bla(123)");

        assert_eq!(p.parse_primary(), Ok(Box::new(ExprAST::Number(1337f64))));

        assert_eq!(
            p.parse_primary(),
            Ok(Box::new(ExprAST::Variable("foop".into())))
        );

        assert_eq!(
            p.parse_primary(),
            Ok(Box::new(ExprAST::Call(
                "bla".into(),
                vec![Box::new(ExprAST::Number(123f64))]
            )))
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

        let binexpr_ab = Box::new(ExprAST::Binary(
            '+',
            Box::new(ExprAST::Variable("a".into())),
            Box::new(ExprAST::Variable("b".into())),
        ));

        let binexpr_abc = Box::new(ExprAST::Binary(
            '-',
            binexpr_ab,
            Box::new(ExprAST::Variable("c".into())),
        ));

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

        let binexpr_bc = Box::new(ExprAST::Binary(
            '*',
            Box::new(ExprAST::Variable("b".into())),
            Box::new(ExprAST::Variable("c".into())),
        ));

        let binexpr_abc = Box::new(ExprAST::Binary(
            '+',
            Box::new(ExprAST::Variable("a".into())),
            binexpr_bc,
        ));

        assert_eq!(p.parse_expression(), Ok(binexpr_abc));
    }
}
