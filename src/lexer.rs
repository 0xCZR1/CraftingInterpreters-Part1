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

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

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

    /* COMMENT */
    Comment,

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

    fn advance(&mut self) -> char {
        let current_byte = self.current;
        self.current += 1;
        self.source.as_bytes()[current_byte] as char
    }

    fn add_token(&mut self, kind: TokenType) {
        self.tokens
            .push(Token::new(kind, "".to_string(), None, self.line));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
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
        self.add_token(TokenType::String);
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[self.current + 1] as char
        }
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.current += 1;
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        self.add_token(TokenType::Number);
    }

    fn is_digit(&self, c: char) -> bool {
        println!("{}", c);
        c >= '0' && c <= '9'
    }

    fn is_alpha(&self, c: char) -> bool {
        c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let text: &str = (&self.source[self.start..self.current]);
        let text_type: TokenType = KEYWORDS.get(text).copied().unwrap_or(TokenType::Identifier);
        self.add_token(text_type);
    }

    fn scan_token(&mut self) -> Result<(), ()> {
        let mut c: char = self.advance();
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen);
                Ok(())
            }
            ')' => {
                self.add_token(TokenType::RightParen);
                Ok(())
            }
            '{' => {
                self.add_token(TokenType::LeftBrace);
                Ok(())
            }
            '}' => {
                self.add_token(TokenType::RightBrace);
                Ok(())
            }
            ',' => {
                self.add_token(TokenType::Comma);
                Ok(())
            }
            '.' => {
                self.add_token(TokenType::Dot);
                Ok(())
            }
            '-' => {
                self.add_token(TokenType::Minus);
                Ok(())
            }
            '+' => {
                self.add_token(TokenType::Plus);
                Ok(())
            }
            ';' => {
                self.add_token(TokenType::Semicolon);
                Ok(())
            }
            '*' => {
                self.add_token(TokenType::Star);
                Ok(())
            }
            '!' => {
                if self.current < self.source.len()
                    && self.source.as_bytes()[(self.current) as usize] as char == '='
                {
                    self.current += 1;
                    self.add_token(TokenType::BangEqual);
                    Ok(())
                } else {
                    self.add_token(TokenType::Bang);
                    Ok(())
                }
            }
            '=' => {
                if self.current < self.source.len()
                    && self.source.as_bytes()[(self.current) as usize] as char == '='
                {
                    self.current += 1;
                    self.add_token(TokenType::EqualEqual);
                    Ok(())
                } else {
                    self.add_token(TokenType::Equal);
                    Ok(())
                }
            }
            '<' => {
                if self.current < self.source.len()
                    && self.source.as_bytes()[(self.current) as usize] as char == '='
                {
                    self.current += 1;
                    self.add_token(TokenType::LessEqual);
                    Ok(())
                } else {
                    self.add_token(TokenType::Less);
                    Ok(())
                }
            }
            '>' => {
                if self.current < self.source.len()
                    && self.source.as_bytes()[(self.current) as usize] as char == '='
                {
                    self.current += 1;
                    self.add_token(TokenType::GreaterEqual);
                    Ok(())
                } else {
                    self.add_token(TokenType::Greater);
                    Ok(())
                }
            }
            '/' => {
                if self.current < self.source.len()
                    && self.source.as_bytes()[(self.current) as usize] as char == '/'
                {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.current += 1;
                    }
                    Ok(())
                } else {
                    self.add_token(TokenType::Slash);
                    Ok(())
                }
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
            VAL => {
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
            _ => {
                let message = "Unexpected identifier";
                Err(println!(
                    "Line: {} found '{}' which is {}",
                    self.line, c, message
                ))
            }
        }
    }

    pub fn scan(&mut self) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        &self.tokens
    }
}
