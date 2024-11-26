pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
use convert::MaybeInto;
use core::str;
use std::sync::atomic::AtomicUsize;
use token::{
    KeywordType, LiteralValue, StringIdentifier, StringValue, Token, TokenCharacter, TokenType,
};

#[derive(Default, Debug)]
pub struct Indices {
    pub start: usize,
    pub current: AtomicUsize,
    pub line: AtomicUsize,
}
pub struct TokenSource {
    pub source: String,
    pub chars: Vec<char>,
    pub indices: Indices,
}
fn is_digit(c: &char) -> bool {
    c.is_ascii_digit()
}
fn is_alpha(c: &char) -> bool {
    c.is_ascii_alphabetic() || c.eq(&'_')
}
fn is_alpha_numeric(c: &char) -> bool {
    is_digit(c) || is_alpha(c)
}
fn evaluate_keyword(s: &str) -> Result<KeywordType, String> {
    s.maybe_into().ok_or(s.into())
}
impl TokenSource {
    fn new(source: &impl ToString) -> Self {
        let source = source.to_string();
        let chars = source.chars().collect();
        Self {
            source,
            chars,
            indices: Default::default(),
        }
    }
    fn make_token(&self, r#type: TokenType, literal: Option<LiteralValue>) -> Option<Token> {
        self.source
            .get(
                self.indices.start
                    ..self
                        .indices
                        .current
                        .load(std::sync::atomic::Ordering::Acquire),
            )
            .map(|lexeme| {
                Token::new(
                    r#type,
                    lexeme,
                    literal,
                    self.indices.line.load(std::sync::atomic::Ordering::Acquire),
                )
            })
    }
    fn add_token(&self, tokens: &mut Vec<Token>, r#type: TokenType, literal: Option<LiteralValue>) {
        if let Some(token) = self.make_token(r#type, literal) {
            tokens.push(token)
        }
    }
    pub fn advance(&self) -> Option<&char> {
        self.chars.get(
            self.indices
                .current
                .fetch_add(1, std::sync::atomic::Ordering::Acquire),
        )
    }
    pub fn is_at_end(&self) -> bool {
        self.indices
            .current
            .load(std::sync::atomic::Ordering::Acquire)
            >= self.chars.len()
    }

    pub fn r#match(&self, expected: impl PartialEq<char>) -> bool {
        if self.is_at_end() {
            return false;
        }
        let index = self
            .indices
            .current
            .load(std::sync::atomic::Ordering::Acquire);
        let equals_expected = |c: &char| expected.eq(c);
        if self.chars.get(index).is_some_and(equals_expected) {
            self.indices
                .current
                .store(index + 1, std::sync::atomic::Ordering::Release);
            true
        } else {
            false
        }
    }
    pub fn peek(&self) -> Option<&char> {
        if self.is_at_end() {
            Some(&'\0')
        } else {
            self.chars.get(
                self.indices
                    .current
                    .load(std::sync::atomic::Ordering::Acquire),
            )
        }
    }
    pub fn peek_next(&self) -> Option<&char> {
        let next_index = self
            .indices
            .current
            .load(std::sync::atomic::Ordering::Acquire)
            + 1;
        if next_index >= self.chars.len() {
            Some(&'\0')
        } else {
            self.chars.get(next_index)
        }
    }

    pub fn string(&self) -> Option<Token> {
        let mut _current;
        let mut _previous;
        let mut line = 0;
        while {
            _current = self.peek();
            _current.is_some_and(|c| c.ne(&'"')) && !self.is_at_end()
        } {
            if _current.is_some_and(|c| c.eq(&'\n')) {
                line = self
                    .indices
                    .line
                    .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                    + 1;
            }
            _previous = self.advance();
        }

        if self.is_at_end() {
            eprintln!("{} Unterminated string", line);
            return None;
        }

        // The closing "
        _previous = _current;
        _current = self.advance();

        let literal_string = self.source.get(
            // Trim the surrounding quoates
            self.indices.start + 1
                ..self
                    .indices
                    .current
                    .load(std::sync::atomic::Ordering::Acquire)
                    - 1,
        );

        let make_token = |s: &str| {
            self.make_token(
                TokenType::String,
                Some(LiteralValue::String(StringValue(s.into()))),
            )
        };
        literal_string.and_then(make_token)
    }
    pub fn identifier(&self) -> Option<Token> {
        let mut _current;
        let mut _previous;
        while {
            _current = self.peek();
            _current.is_some_and(is_alpha_numeric)
        } {
            _previous = self.advance();
        }
        let text = self.source.get(
            self.indices.start
                ..self
                    .indices
                    .current
                    .load(std::sync::atomic::Ordering::Acquire),
        );
        match text
            .map(KeywordType::try_from)
            .transpose()
            .map_err(LiteralValue::Identifier)
        {
            Ok(keyword_type) => keyword_type
                .map(TokenType::Keyword)
                .and_then(|r#type| self.make_token(r#type, None)),
            Err(identifier) => self.make_token(TokenType::Identifier, Some(identifier)),
        }

        // self.source
        //     .get(
        //         self.indices.start
        //             ..self
        //                 .indices
        //                 .current
        //                 .load(std::sync::atomic::Ordering::Acquire),
        //     )
        //     .map(ToString::to_string)
        //     .map(StringIdentifier)
    }
    pub fn number(&self) -> Option<Token> {
        let mut _current;
        let mut _previous;
        while {
            _current = self.peek();
            _current.is_some_and(|c| c.is_numeric())
        } {
            _previous = self.advance();
        }

        if _current.is_some_and(|c| c.eq(&'.')) && self.peek_next().is_some_and(|c| c.is_numeric())
        {
            // Consume the '.'
            _previous = _current;
            _current = self.advance();
        }
        let literal = self
            .source
            .get(
                self.indices.start
                    ..self
                        .indices
                        .current
                        .load(std::sync::atomic::Ordering::Acquire),
            )
            .and_then(|sub| {
                let sub: Result<f32, <f32 as str::FromStr>::Err> = sub.parse();
                sub.ok()
            })
            .map(LiteralValue::Number);
        self.make_token(TokenType::Number, literal)
    }
}
pub struct Tokenizer {
    pub source: TokenSource,
    pub indices: Indices,
    pub len: usize,
}
impl Iterator for Tokenizer {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        // let line = self
        //     .source
        //     .indices
        //     .line
        //     .load(std::sync::atomic::Ordering::Acquire);
        let next = self.source.advance()?;
        let next = TokenCharacter::try_from(next);
        match next {
            Ok(TokenCharacter::Type(r#type)) => self.source.make_token(r#type, None),
            Ok(TokenCharacter::Comparator(comparator)) => self
                .source
                .make_token(comparator.r#match(self.source.r#match('=')), None),
            Ok(TokenCharacter::Delimiter(delimiter)) => match delimiter {
                token::DelimiterType::Comment => {
                    if self.source.r#match('/') {
                        while self.source.peek().is_some_and(|c| c.ne(&'\n'))
                            && !self.source.is_at_end()
                        {
                            self.source
                                .indices
                                .current
                                .fetch_add(1, std::sync::atomic::Ordering::Acquire);
                        }
                        None
                    } else {
                        self.source.make_token(TokenType::Slash, None)
                    }
                }
                token::DelimiterType::Literal(literal) => match literal {
                    token::LiteralType::String => self.source.string(),
                    token::LiteralType::Number => self.source.number(),
                    token::LiteralType::Identifier => self.source.identifier(),
                },
            },
            Ok(TokenCharacter::Newline) => {
                self.source
                    .indices
                    .line
                    .fetch_add(1, std::sync::atomic::Ordering::Acquire);
                None
            }
            Ok(_) => None,
            Err(token::Unexpected(unexpected)) => {
                eprintln!(
                    "{} Unexpected character: {}",
                    self.source
                        .indices
                        .line
                        .load(std::sync::atomic::Ordering::Acquire),
                    unexpected
                );
                None
            } // '!' => self.source.make_token(
              //     self.source
              //         .r#match('=')
              //         .then_some(TokenType::BangEqual)
              //         .unwrap_or(TokenType::Bang),
              //     None,
              // ),
              // '=' => self.source.make_token(
              //     self.source
              //         .r#match('=')
              //         .then_some(TokenType::EqualEqual)
              //         .unwrap_or(TokenType::Equal),
              //     None,
              // ),
              // '<' => self.source.make_token(
              //     self.source
              //         .r#match('=')
              //         .then_some(TokenType::LessEqual)
              //         .unwrap_or(TokenType::Less),
              //     None,
              // ),
              // '>' => self.source.make_token(
              //     self.source
              //         .r#match('=')
              //         .then_some(TokenType::GreaterEqual)
              //         .unwrap_or(TokenType::Greater),
              //     None,
              // ),
              // '/' => {
              //     if self.source.r#match('/') {
              //         while self.source.peek().is_some_and(|c| c.ne(&'\n'))
              //             && !self.source.is_at_end()
              //         {
              //             self.source
              //                 .indices
              //                 .current
              //                 .fetch_add(1, std::sync::atomic::Ordering::Acquire);
              //         }
              //         None
              //     } else {
              //         self.source.make_token(TokenType::Slash, None)
              //     }
              // }
              // '"' => self.source.string(),
              // ' ' | '\r' | '\t' => None,
              // '\n' => {
              //     self.source
              //         .indices
              //         .line
              //         .fetch_add(1, std::sync::atomic::Ordering::Acquire);
              //     None
              // }
              // c if c.is_numeric() => self.source.number(),
              // c if c.is_alphabetic() => self.source.identifier(),
              // 'o' if self.source.r#match('r')
              //     && self.source.peek_next().is_some_and(|c| c.is_whitespace()) =>
              // {
              //     self.source.make_token(TokenType::Or, None)
              // }
              // unexpected => {
              //     eprintln!(
              //         "{} Unexpected character: {}",
              //         self.source
              //             .indices
              //             .line
              //             .load(std::sync::atomic::Ordering::Acquire),
              //         unexpected
              //     );
              //     None
              // }
        }
    }
}
impl Tokenizer {
    pub fn new(source: &impl ToString) -> Self {
        let source = TokenSource::new(source);

        let len = source.chars.len();
        Self {
            source,
            indices: Indices::default(),
            len,
        }
    }
    pub fn scan_token(&mut self, tokens: &mut Vec<Token>) {
        // if let Some(next) = self.source.advance() {
        //     match next {
        //         '(' => {
        //             self.source
        //                 .add_token(tokens, TokenType::Paren(token::Direction::Left), None)
        //         }
        //         ')' => {
        //             self.source
        //                 .add_token(tokens, TokenType::Paren(token::Direction::Right), None)
        //         }
        //         '{' => {
        //             self.source
        //                 .add_token(tokens, TokenType::Brace(token::Direction::Left), None)
        //         }
        //         '}' => {
        //             self.source
        //                 .add_token(tokens, TokenType::Brace(token::Direction::Right), None)
        //         }
        //         ',' => self.source.add_token(tokens, TokenType::Comma, None),
        //         '.' => self.source.add_token(tokens, TokenType::Dot, None),
        //         '-' => self.source.add_token(tokens, TokenType::Minus, None),
        //         '+' => self.source.add_token(tokens, TokenType::Plus, None),
        //         ';' => self.source.add_token(tokens, TokenType::Semicolon, None),
        //         '*' => self.source.add_token(tokens, TokenType::Star, None),
        //         '!' => self.source.add_token(
        //             tokens,
        //             self.source
        //                 .r#match('=')
        //                 .then_some(TokenType::BangEqual)
        //                 .unwrap_or(TokenType::Bang),
        //             None,
        //         ),
        //         '=' => self.source.add_token(
        //             tokens,
        //             self.source
        //                 .r#match('=')
        //                 .then_some(TokenType::EqualEqual)
        //                 .unwrap_or(TokenType::Equal),
        //             None,
        //         ),
        //         '<' => self.source.add_token(
        //             tokens,
        //             self.source
        //                 .r#match('=')
        //                 .then_some(TokenType::LessEqual)
        //                 .unwrap_or(TokenType::Less),
        //             None,
        //         ),
        //         '>' => self.source.add_token(
        //             tokens,
        //             self.source
        //                 .r#match('=')
        //                 .then_some(TokenType::GreaterEqual)
        //                 .unwrap_or(TokenType::Greater),
        //             None,
        //         ),
        //         '/' => {
        //             if self.source.r#match('/') {
        //                 while self.source.peek().is_some_and(|c| c.ne(&'\n'))
        //                     && !self.source.is_at_end()
        //                 {
        //                     self.source
        //                         .indices
        //                         .current
        //                         .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        //                 }
        //             } else {
        //                 self.source.add_token(tokens, TokenType::Slash, None);
        //             }
        //         }
        //         '"' => self.source.add_token(
        //             tokens,
        //             TokenType::String,
        //             self.source.string().map(LiteralValue::String),
        //         ),
        //         c if c.is_numeric() => self.source.add_token(
        //             tokens,
        //             TokenType::Number,
        //             self.source.number().map(LiteralValue::Number),
        //         ),
        //         ' ' | '\r' | '\t' => {}
        //         '\n' => {
        //             self.source
        //                 .indices
        //                 .line
        //                 .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        //         }
        //         unexpected => {
        //             eprintln!(
        //                 "{} Unexpected character: {}",
        //                 self.source
        //                     .indices
        //                     .line
        //                     .load(std::sync::atomic::Ordering::Acquire),
        //                 unexpected
        //             );
        //         }
        //     }
        // };
        if let Some(token) = self.next() {
            tokens.push(token)
        }
    }
    // fn advance(&mut self) -> Option<&char> {
    //     let current = self.source.get(self.indices.current)?;
    //     self.indices.current += 1;
    //     Some(current)
    // }

    // pub fn is_at_end(&self) -> bool {
    //     self.indices.current >= self.len
    // }
}
pub struct Scanner {
    pub source: String,
    pub tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: impl ToString) -> Self {
        Self {
            source: source.to_string(),
            tokens: Default::default(),
        }
    }
    // fn add_token(
    //     &self,
    //     indices: &Indices,
    //     tokens: &mut Vec<Token>,
    //     r#type: TokenType,
    //     literal: Option<LiteralValue>,
    // ) {
    //     let lexeme = self
    //         .source
    //         .get(indices.start..indices.current)
    //         .unwrap_or_default();
    //     tokens.push(Token::new(r#type, lexeme, literal, indices.line))
    // }
    // fn add_string(&self, tokenizer: &mut Tokenizer, tokens: &mut Vec<Token>) {
    //     while tokenizer.peek().is_some_and(|c| c.ne(&'"')) && !tokenizer.is_at_end() {
    //         if tokenizer.peek().is_some_and(|c| c.eq(&'\n')) {
    //             tokenizer.indices.line += 1;
    //         }
    //         let _ = tokenizer.advance();
    //     }

    //     if tokenizer.is_at_end() {
    //         eprintln!("{} Unterminated string", tokenizer.indices.line);
    //         return;
    //     }

    //     // The closing "
    //     let _ = tokenizer.advance();

    //     // Trim the surrounding quoates
    //     let Some(value) = self
    //         .source
    //         .get(tokenizer.indices.start + 1..tokenizer.indices.current - 1)
    //     else {
    //         return;
    //     };
    //     self.add_token(
    //         &tokenizer.indices,
    //         tokens,
    //         TokenType::String,
    //         Some(LiteralValue::String(StringValue(value.to_string()))),
    //     )
    // }
    // fn advance(&self, tokenizer: &mut Tokenizer) -> Option<TokenType> {
    //     tokenizer.next()
    // }
    // fn scan_token(&self, tokenizer: &mut Tokenizer, tokens: &mut Vec<Token>) {
    //     match tokenizer.next() {
    //         Some(TokenType::String) => self.add_string(tokenizer, tokens),
    //         Some(r#type) => self.add_token(&tokenizer.indices, tokens, r#type, None),
    //         None => {}
    //     }
    // }
    pub fn scan_tokens(&self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut tokenizer = Tokenizer::new(&self.source);
        while !tokenizer.source.is_at_end() {
            tokenizer.source.indices.start = tokenizer
                .source
                .indices
                .current
                .load(std::sync::atomic::Ordering::Acquire);
            tokenizer.scan_token(&mut tokens);
        }
        tokens.push(Token::new(
            token::TokenType::Eof,
            "",
            None,
            tokenizer
                .source
                .indices
                .line
                .load(std::sync::atomic::Ordering::Acquire),
        ));
        // tokenizer
        //     .source
        //     .add_token(&mut tokens, token::TokenType::Eof, None);
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let source = "var language = \"lox\";";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        assert!(!tokens.is_empty());
        tokens.iter().for_each(|token| println!("{token}"))
    }
}
