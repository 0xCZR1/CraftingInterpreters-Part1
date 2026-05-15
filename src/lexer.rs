//! Lexical analysis for LoX
//! The scanner walks a source one byte at a time and consumes it, mapping the token value.
//! Currently this only supports ASCII. While NON-ASCII bytes outside string literals are consumed as errors via 'RoX' and do not stop scanning,
//! the scanner always consumes its full input, so callers see all errors at once.

use super::rox::RoX;
use std::collections::HashMap;
use std::sync::LazyLock;

/* STATICS */

static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    let mut keyword_map = HashMap::new();
    keyword_map.insert("and", TokenType::And);
    keyword_map.insert("class", TokenType::Class);
    keyword_map.insert("else", TokenType::Else);
    keyword_map.insert("false", TokenType::False);
    keyword_map.insert("for", TokenType::For);
    keyword_map.insert("fun", TokenType::Fun);
    keyword_map.insert("if", TokenType::If);
    keyword_map.insert("nil", TokenType::Nil);
    keyword_map.insert("or", TokenType::Or);
    keyword_map.insert("print", TokenType::Print);
    keyword_map.insert("return", TokenType::Return);
    keyword_map.insert("super", TokenType::Super);
    keyword_map.insert("this", TokenType::This);
    keyword_map.insert("true", TokenType::True);
    keyword_map.insert("var", TokenType::Var);
    keyword_map.insert("while", TokenType::While);
    keyword_map
});

/// A parsed runtime value attached to certain tokens
/// Only String and Number currently return, the others are reserved for later stages.
#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

/// A single lexical token produced by the scanner
/// 'lexeme' is the exact slice of the source scanned
/// 'literal' is the parsed runtime value where applicable
#[derive(Debug)]
pub struct Token {
    kind: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /* SINGLE CHARACTER TOKENS */
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    /* ONE OR TWO CHARACTER TOKENS */
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    /* LITERALS */
    Identifier,
    String,
    Number,

    /* Keywords */
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, literal: Option<Literal>, line: u64) -> Self {
        Self {
            kind,
            lexeme,
            literal,
            line,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:#?} {:#?} {:#?} {:#?}",
            self.kind, self.lexeme, self.literal, self.line
        )
    }
}
#[derive(Debug)]
pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u64,
}

impl Lexer {
    pub fn new(source: String, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    // ------- Helpers (lookahead and consumption)

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() as usize
    }

    fn peek(&self) -> char {
        let current_byte = self.current;
        if self.is_at_end() {
            '\0'
        } else {
            self.source.as_bytes()[current_byte] as char
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[self.current + 1] as char
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        let current_byte = self.current;
        self.current += 1;
        self.source.as_bytes()[current_byte] as char
    }

    // ------- Multi-character token scanners

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.advance();
            } else {
                self.advance();
            };
        }

        if self.is_at_end() {
            let mut rox_err = RoX::new();
            rox_err.report_error(self.line, "Unterminated string");
            return;
        }

        self.advance();

        //Trim surrounding quotes
        let value: String = (&self.source[self.start + 1..self.current - 1]).to_string();
        self.add_token(TokenType::String, Some(Literal::String(value)));
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        let value: f64 = (&self.source[self.start..self.current]).parse().unwrap();
        self.add_token(TokenType::Number, Some(Literal::Number(value)));
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let text: &str = &self.source[self.start..self.current];
        let text_type: TokenType = KEYWORDS.get(text).copied().unwrap_or(TokenType::Identifier);
        self.add_token(text_type, None);
    }

    fn block_comment(&mut self) {
        self.advance();

        loop {
            if self.is_at_end() {
                let mut rox_err = RoX::new();
                rox_err.report_error(self.line, "Unterminated block comment");
                return;
            }
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance(); //Consume */
                return;
            }
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
    }

    /// ------- Helper functions for the multi-token scanning functions

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alpha(&self, c: char) -> bool {
        c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    /// ------- Dispatch

    fn add_token(&mut self, kind: TokenType, literal: Option<Literal>) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(kind, lexeme, literal, self.line));
    }

    fn scan_token(&mut self) -> Result<(), ()> {
        let c: char = self.advance();
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen, None);
                Ok(())
            }
            ')' => {
                self.add_token(TokenType::RightParen, None);
                Ok(())
            }
            '{' => {
                self.add_token(TokenType::LeftBrace, None);
                Ok(())
            }
            '}' => {
                self.add_token(TokenType::RightBrace, None);
                Ok(())
            }
            ',' => {
                self.add_token(TokenType::Comma, None);
                Ok(())
            }
            '.' => {
                self.add_token(TokenType::Dot, None);
                Ok(())
            }
            '-' => {
                self.add_token(TokenType::Minus, None);
                Ok(())
            }
            '+' => {
                self.add_token(TokenType::Plus, None);
                Ok(())
            }
            ';' => {
                self.add_token(TokenType::Semicolon, None);
                Ok(())
            }
            '*' => {
                self.add_token(TokenType::Star, None);
                Ok(())
            }
            '!' => {
                let kind = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(kind, None);
                Ok(())
            }
            '=' => {
                let kind = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(kind, None);
                Ok(())
            }
            '<' => {
                let kind = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(kind, None);
                Ok(())
            }
            '>' => {
                let kind = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(kind, None);
                Ok(())
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    self.block_comment();
                } else {
                    self.add_token(TokenType::Slash, None);
                }
                Ok(())
            }
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            '"' => {
                self.string();
                Ok(())
            }
            _ => {
                if self.is_digit(c) {
                    self.number();
                    Ok(())
                } else if self.is_alpha(c) {
                    self.identifier();
                    Ok(())
                } else {
                    let message = "Unexpected character";
                    Err(println!(
                        "Line: {} found '{}' which is {}",
                        self.line, c, message
                    ))
                }
            }
        }
    }
    /// Scans the source and returns the tokens
    pub fn scan(&mut self) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            let _ = self.scan_token();
        }
        &self.tokens
    }
}

/* UNIT TESTS */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiline_string_terminates() {
        let mut lexer = Lexer::new("\"a\nb\"".to_string(), vec![]);
        lexer.scan();
        assert_eq!(lexer.line, 2);
    }

    #[test]
    fn two_operators() {
        let mut lexer = Lexer::new("!= == >= <=".to_string(), vec![]);
        let kinds: Vec<_> = lexer.scan().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenType::BangEqual,
                TokenType::EqualEqual,
                TokenType::GreaterEqual,
                TokenType::LessEqual,
            ]
        )
    }

    #[test]
    fn single_operator() {
        let mut lexer = Lexer::new("( ) { } , . - + ; / * > < =".to_string(), vec![]);
        let kind: Vec<_> = lexer.scan().iter().map(|t| t.kind).collect();
        assert_eq!(
            kind,
            vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Comma,
                TokenType::Dot,
                TokenType::Minus,
                TokenType::Plus,
                TokenType::Semicolon,
                TokenType::Slash,
                TokenType::Star,
                TokenType::Greater,
                TokenType::Less,
                TokenType::Equal,
            ]
        );
    }

    #[test]
    fn keyword_vs_identifier() {
        let mut lexer = Lexer::new("var orchid or".to_string(), vec![]);
        let kind: Vec<_> = lexer.scan().iter().map(|t| t.kind).collect();
        assert_eq!(
            kind,
            vec![TokenType::Var, TokenType::Identifier, TokenType::Or]
        );
    }

    #[test]
    fn keyword() {
        let mut lexer = Lexer::new(
            "and class else false fun for if nil or print return super this true var while"
                .to_string(),
            vec![],
        );
        let kind: Vec<_> = lexer.scan().iter().map(|t| t.kind).collect();
        assert_eq!(
            kind,
            vec![
                TokenType::And,
                TokenType::Class,
                TokenType::Else,
                TokenType::False,
                TokenType::Fun,
                TokenType::For,
                TokenType::If,
                TokenType::Nil,
                TokenType::Or,
                TokenType::Print,
                TokenType::Return,
                TokenType::Super,
                TokenType::This,
                TokenType::True,
                TokenType::Var,
                TokenType::While,
            ]
        );
    }

    #[test]
    fn newline_test() {
        let mut lexer = Lexer::new("test \nb \nc \nd".to_string(), vec![]);
        let lines: Vec<_> = lexer.scan().iter().map(|l| l.line).collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn line_comment_produces_no_token() {
        let mut lexer = Lexer::new("// just a comment".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn line_comment_stops_at_newline() {
        let mut lexer = Lexer::new("// comment\n+".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenType::Plus);
        assert_eq!(tokens[0].line, 2);
    }

    #[test]
    fn code_before_line_comment_still_tokenizes() {
        let mut lexer = Lexer::new("var x // trailing".to_string(), vec![]);
        let tokens = lexer.scan();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![TokenType::Var, TokenType::Identifier]);
    }

    #[test]
    fn slash_is_division_not_comment() {
        let mut lexer = Lexer::new("1 / 2".to_string(), vec![]);
        let tokens = lexer.scan();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![TokenType::Number, TokenType::Slash, TokenType::Number]
        );
    }

    #[test]
    fn block_comment_produces_no_token() {
        let mut lexer = Lexer::new("/* hello */".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn empty_block_comment() {
        let mut lexer = Lexer::new("/**/".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn block_comment_counts_internal_newlines() {
        let mut lexer = Lexer::new("/* a\nb\nc */".to_string(), vec![]);
        lexer.scan();
        assert_eq!(lexer.line, 3);
    }

    #[test]
    fn code_around_block_comment() {
        let mut lexer = Lexer::new("a /* mid */ b".to_string(), vec![]);
        let tokens = lexer.scan();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![TokenType::Identifier, TokenType::Identifier]);
    }

    #[test]
    fn block_comment_with_lone_star_inside() {
        // The `*` followed by ` ` (not `/`) must not terminate the comment.
        let mut lexer = Lexer::new("/* foo * bar */ +".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenType::Plus);
    }

    #[test]
    fn block_comment_with_lone_slash_inside() {
        // A `/` not preceded by `*` must not terminate.
        let mut lexer = Lexer::new("/* a / b */ +".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenType::Plus);
    }

    #[test]
    fn two_block_comments_in_a_row() {
        let mut lexer = Lexer::new("/* one */ /* two */ +".to_string(), vec![]);
        let tokens = lexer.scan();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenType::Plus);
    }

    #[test]
    fn line_comment_after_code_on_same_line() {
        let mut lexer = Lexer::new("var x; // declare\nvar y;".to_string(), vec![]);
        let tokens = lexer.scan();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenType::Var,
                TokenType::Identifier,
                TokenType::Semicolon,
                TokenType::Var,
                TokenType::Identifier,
                TokenType::Semicolon,
            ]
        );
    }
}
