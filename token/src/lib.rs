use convert::{MaybeFrom, MaybeInto};
use core::fmt;
use std::ops;
pub enum Direction {
    Left,
    Right,
}
pub enum TokenType {
    // Single-character tokens
    Paren(Direction),
    Brace(Direction),
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Comparator { r#type: ComparatorType, equal: bool },
    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    Keyword(KeywordType),

    Eof,
}
pub enum ComparatorType {
    Bang,
    Equal,
    Greater,
    Less,
}
impl fmt::Display for ComparatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            // One or two character token.into(),
            Self::Bang => "BANG",
            Self::Equal => "EQUAL",
            Self::Greater => "GREATER",
            Self::Less => "LESS",
        })
    }
}
impl ComparatorType {
    pub fn r#match(self, equal: bool) -> TokenType {
        TokenType::Comparator {
            r#type: self,
            equal,
        }
    }
}
// pub enum LiteralType {

// }
pub enum KeywordType {
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
}
impl MaybeFrom<&str> for KeywordType {
    fn maybe_from(value: &str) -> Option<Self> {
        match value.as_ref() {
            "and" => Some(KeywordType::And),
            "class" => Some(KeywordType::Class),
            "else" => Some(KeywordType::Else),
            "false" => Some(KeywordType::False),
            "for" => Some(KeywordType::For),
            "fun" => Some(KeywordType::Fun),
            "if" => Some(KeywordType::If),
            "nil" => Some(KeywordType::Nil),
            "or" => Some(KeywordType::Or),
            "print" => Some(KeywordType::Print),
            "return" => Some(KeywordType::Return),
            "super" => Some(KeywordType::Super),
            "this" => Some(KeywordType::This),
            "true" => Some(KeywordType::True),
            "var" => Some(KeywordType::Var),
            "while" => Some(KeywordType::While),
            _ => None,
        }
    }
}
impl TryFrom<&str> for KeywordType {
    type Error = StringIdentifier;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .maybe_into()
            .ok_or(value.into())
            .map_err(StringIdentifier)
    }
}
impl fmt::Display for KeywordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            // Keyword,
            Self::And => "AND",
            Self::Class => "CLASS",
            Self::Else => "ELSE",
            Self::False => "FALSE",
            Self::Fun => "FUN",
            Self::For => "FOR",
            Self::If => "IF",
            Self::Nil => "NIL",
            Self::Or => "OR",
            Self::Print => "PRINT",
            Self::Return => "RETURN",
            Self::Super => "SUPER",
            Self::This => "THIS",
            Self::True => "TRUE",
            Self::Var => "VAR",
            Self::While => "WHILE",
        })
    }
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            // Single-character tokens
            Self::Paren(Direction::Left) => "LEFT_PAREN".into(),
            Self::Paren(Direction::Right) => "RIGHT_PAREN".into(),
            Self::Brace(Direction::Left) => "LEFT_BRACE".into(),
            Self::Brace(Direction::Right) => "RIGHT_BRACE".into(),
            Self::Comma => "COMMA".into(),
            Self::Dot => "DOT".into(),
            Self::Minus => "MINUS".into(),
            Self::Plus => "PLUS".into(),
            Self::Semicolon => "SEMICOLON".into(),
            Self::Slash => "SLASH".into(),
            Self::Star => "STAR".into(),

            // One or two character token.into(),
            Self::Comparator {
                r#type,
                equal: false,
            } => format!("{type}"),
            Self::Comparator {
                r#type,
                equal: true,
            } => format!("{type}_EQUAL"),
            // Literal.into(),
            Self::Identifier => "IDENTIFIER".into(),
            Self::String => "STRING".into(),
            Self::Number => "NUMBER".into(),

            // Keyword,
            Self::Keyword(keyword) => keyword.to_string(),

            Self::Eof => "EOF".into(),
        })
    }
}
pub enum LiteralType {
    String,
    Identifier,
    Number,
}
pub enum DelimiterType {
    Comment,
    Literal(LiteralType),
}
pub enum TokenCharacter {
    Type(TokenType),
    Comparator(ComparatorType),
    Delimiter(DelimiterType),
    Whitespace,
    Tab,
    CarriageReturn,
    Newline,
}
pub struct Unexpected<'a>(pub &'a char);
impl<'a> TryFrom<&'a char> for TokenCharacter {
    type Error = Unexpected<'a>;
    fn try_from(value: &char) -> Result<Self, Unexpected<'_>> {
        match value {
            '(' => Ok(TokenCharacter::Type(TokenType::Paren(Direction::Left))),
            ')' => Ok(TokenCharacter::Type(TokenType::Paren(Direction::Right))),
            '{' => Ok(TokenCharacter::Type(TokenType::Brace(Direction::Left))),
            '}' => Ok(TokenCharacter::Type(TokenType::Brace(Direction::Right))),
            ',' => Ok(TokenCharacter::Type(TokenType::Comma)),
            '.' => Ok(TokenCharacter::Type(TokenType::Dot)),
            '-' => Ok(TokenCharacter::Type(TokenType::Minus)),
            '+' => Ok(TokenCharacter::Type(TokenType::Plus)),
            ';' => Ok(TokenCharacter::Type(TokenType::Semicolon)),
            '*' => Ok(TokenCharacter::Type(TokenType::Star)),
            '!' => Ok(TokenCharacter::Comparator(ComparatorType::Bang)),
            '=' => Ok(TokenCharacter::Comparator(ComparatorType::Equal)),
            '<' => Ok(TokenCharacter::Comparator(ComparatorType::Less)),
            '>' => Ok(TokenCharacter::Comparator(ComparatorType::Greater)),
            ' ' => Ok(TokenCharacter::Whitespace),
            '\r' => Ok(TokenCharacter::CarriageReturn),
            '\t' => Ok(TokenCharacter::Tab),
            '\n' => Ok(TokenCharacter::Newline),
            '/' => Ok(TokenCharacter::Delimiter(DelimiterType::Comment)),
            '"' => Ok(TokenCharacter::Delimiter(DelimiterType::Literal(
                LiteralType::String,
            ))),
            c if c.is_numeric() => Ok(TokenCharacter::Delimiter(DelimiterType::Literal(
                LiteralType::Number,
            ))),
            c if c.is_alphabetic() => Ok(TokenCharacter::Delimiter(DelimiterType::Literal(
                LiteralType::Identifier,
            ))),
            unexpected => Err(Unexpected(unexpected)),
        }
    }
}
pub trait PrimaryLiteral: Literal {
    fn to_literal_value(&self) -> LiteralValue;
}
pub trait Literal: fmt::Display {}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct StringValue(pub String);
impl ops::Deref for StringValue {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl fmt::Debug for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}
impl From<&str> for StringValue {
    fn from(value: &str) -> Self {
        StringValue(value.into())
    }
}
impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self)
    }
}
impl Literal for StringValue {}
impl PrimaryLiteral for StringValue {
    fn to_literal_value(&self) -> LiteralValue {
        LiteralValue::String(self.clone())
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct StringIdentifier(pub String);
impl ops::Deref for StringIdentifier {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl fmt::Display for StringIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self)
    }
}
impl Literal for StringIdentifier {}
impl PrimaryLiteral for StringIdentifier {
    fn to_literal_value(&self) -> LiteralValue {
        LiteralValue::Identifier(self.clone())
    }
}
impl Literal for f32 {}
impl PrimaryLiteral for f32 {
    fn to_literal_value(&self) -> LiteralValue {
        LiteralValue::Number(*self)
    }
}
pub enum LiteralValue {
    Identifier(StringIdentifier),
    String(StringValue),
    Number(f32),
}
// impl From<Option<Box<dyn Literal>>> for LiteralValue {
//     fn from(value: Option<Box<dyn Literal>>) -> Self {
//         match value {
//             Some(boxed) => boxed.to_literal_value(),
//             None => LiteralValue::Null,
//         }
//     }
// }
// impl fmt::Display for LiteralKind {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.write_str(match self {
//             Self::Identifier => "IDENTIFIER",
//             Self::String => "STRING",
//             Self::Number => "NUMBER",
//         })
//     }
// }
#[derive(Default)]
/// We use Option<Box<dyn Literal>>
/// To leverage
///  - Nullability
///  - Polymorphism
pub struct NullableLiteral(Option<Box<dyn PrimaryLiteral>>);
impl ops::Deref for NullableLiteral {
    type Target = Option<Box<dyn PrimaryLiteral>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl NullableLiteral {
    fn new(literal: impl PrimaryLiteral + 'static) -> Self {
        let boxed: Box<dyn PrimaryLiteral> = Box::new(literal);
        Self(Some(boxed))
    }
}
impl Literal for NullableLiteral {}
impl From<NullableLiteral> for LiteralValue {
    fn from(value: NullableLiteral) -> Self {
        value.into()
    }
}
// impl From<&dyn PrimaryLiteral> for NullableLiteral {
//     fn from(value: &dyn PrimaryLiteral) -> Self {
//         value.to_literal_value()
//     }
// }
impl From<LiteralValue> for NullableLiteral {
    fn from(value: LiteralValue) -> Self {
        match value {
            LiteralValue::Identifier(si) => Self::new(si),
            LiteralValue::String(sv) => Self::new(sv),
            LiteralValue::Number(x) => Self::new(x),
        }
    }
}
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub literal: NullableLiteral,
    pub line: usize,
}
impl Token {
    pub fn new(
        r#type: TokenType,
        lexeme: impl ToString,
        literal: Option<LiteralValue>,
        line: usize,
    ) -> Self {
        Self {
            r#type,
            lexeme: lexeme.to_string(),
            literal: literal.map_or(Default::default(), NullableLiteral::from),
            line,
        }
    }
}
impl fmt::Display for NullableLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(ref boxed) => f.write_fmt(format_args!("{}", boxed)),
            None => f.write_str("null"),
        }
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {}",
            self.r#type, self.lexeme, self.literal
        ))
    }
}
