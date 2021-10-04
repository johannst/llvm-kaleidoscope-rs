#[derive(Debug, PartialEq)]
pub enum Token {
    Eof,
    Def,
    Extern,
    Identifier(String),
    Number(f64),
    Char(char),
    If,
    Then,
    Else,
}

pub struct Lexer<I>
where
    I: Iterator<Item = char>,
{
    input: I,
    last_char: Option<char>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(mut input: I) -> Lexer<I> {
        let last_char = input.next();
        Lexer { input, last_char }
    }

    fn step(&mut self) -> Option<char> {
        self.last_char = self.input.next();
        self.last_char
    }

    /// Lex and return the next token.
    ///
    /// Implement `int gettok();` from the tutorial.
    pub fn gettok(&mut self) -> Token {
        // Eat up whitespaces.
        while matches!(self.last_char, Some(c) if c.is_ascii_whitespace()) {
            self.step();
        }

        // Unpack last char or return EOF.
        let last_char = if let Some(c) = self.last_char {
            c
        } else {
            return Token::Eof;
        };

        // Identifier: [a-zA-Z][a-zA-Z0-9]*
        if last_char.is_ascii_alphabetic() {
            let mut ident = String::new();
            ident.push(last_char);

            while let Some(c) = self.step() {
                if c.is_ascii_alphanumeric() {
                    ident.push(c)
                } else {
                    break;
                }
            }

            match ident.as_ref() {
                "def" => return Token::Def,
                "extern" => return Token::Extern,
                "if" => return Token::If,
                "then" => return Token::Then,
                "else" => return Token::Else,
                _ => {}
            }

            return Token::Identifier(ident);
        }

        // Number: [0-9.]+
        if last_char.is_ascii_digit() || last_char == '.' {
            let mut num = String::new();
            num.push(last_char);

            while let Some(c) = self.step() {
                if c.is_ascii_digit() || c == '.' {
                    num.push(c)
                } else {
                    break;
                }
            }

            let num: f64 = num.parse().unwrap_or_default();
            return Token::Number(num);
        }

        // Eat up comment.
        if last_char == '#' {
            loop {
                match self.step() {
                    Some(c) if c == '\r' || c == '\n' => return self.gettok(),
                    None => return Token::Eof,
                    _ => { /* consume comment */ }
                }
            }
        }

        // Advance last char and return currently last char.
        self.step();
        Token::Char(last_char)
    }
}

#[cfg(test)]
mod test {
    use super::{Lexer, Token};

    #[test]
    fn test_identifier() {
        let mut lex = Lexer::new("a b c".chars());
        assert_eq!(Token::Identifier("a".into()), lex.gettok());
        assert_eq!(Token::Identifier("b".into()), lex.gettok());
        assert_eq!(Token::Identifier("c".into()), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());
    }

    #[test]
    fn test_keyword() {
        let mut lex = Lexer::new("def extern".chars());
        assert_eq!(Token::Def, lex.gettok());
        assert_eq!(Token::Extern, lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());
    }

    #[test]
    fn test_number() {
        let mut lex = Lexer::new("12.34".chars());
        assert_eq!(Token::Number(12.34f64), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());

        let mut lex = Lexer::new(" 1.0   2.0 3.0".chars());
        assert_eq!(Token::Number(1.0f64), lex.gettok());
        assert_eq!(Token::Number(2.0f64), lex.gettok());
        assert_eq!(Token::Number(3.0f64), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());

        let mut lex = Lexer::new("12.34.56".chars());
        assert_eq!(Token::Number(0f64), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());
    }

    #[test]
    fn test_comment() {
        let mut lex = Lexer::new("# some comment".chars());
        assert_eq!(Token::Eof, lex.gettok());

        let mut lex = Lexer::new("abc # some comment \n xyz".chars());
        assert_eq!(Token::Identifier("abc".into()), lex.gettok());
        assert_eq!(Token::Identifier("xyz".into()), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());
    }

    #[test]
    fn test_chars() {
        let mut lex = Lexer::new("a+b-c".chars());
        assert_eq!(Token::Identifier("a".into()), lex.gettok());
        assert_eq!(Token::Char('+'), lex.gettok());
        assert_eq!(Token::Identifier("b".into()), lex.gettok());
        assert_eq!(Token::Char('-'), lex.gettok());
        assert_eq!(Token::Identifier("c".into()), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());
    }

    #[test]
    fn test_whitespaces() {
        let mut lex = Lexer::new("    +a  b      c!    ".chars());
        assert_eq!(Token::Char('+'), lex.gettok());
        assert_eq!(Token::Identifier("a".into()), lex.gettok());
        assert_eq!(Token::Identifier("b".into()), lex.gettok());
        assert_eq!(Token::Identifier("c".into()), lex.gettok());
        assert_eq!(Token::Char('!'), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());

        let mut lex = Lexer::new("\n    a \n\r  b \r \n     c \r\r  \n   ".chars());
        assert_eq!(Token::Identifier("a".into()), lex.gettok());
        assert_eq!(Token::Identifier("b".into()), lex.gettok());
        assert_eq!(Token::Identifier("c".into()), lex.gettok());
        assert_eq!(Token::Eof, lex.gettok());
    }

    #[test]
    fn test_ite() {
        let mut lex = Lexer::new("if then else".chars());
        assert_eq!(Token::If, lex.gettok());
        assert_eq!(Token::Then, lex.gettok());
        assert_eq!(Token::Else, lex.gettok());
    }
}
