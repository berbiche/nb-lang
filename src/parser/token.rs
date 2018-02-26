use phf;

use std::convert;
use std::fmt;

use self::{Keyword::*, ReservedKeyword::*};

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "alias" => Reserved(Alias),
    "array" => Reserved(Array),
    "break" => Reserved(Break),
    "case" => Reserved(Case),
    "class" => Reserved(Class),
    "const" => Const,
    "continue" => Reserved(Continue),
    "do" => Reserved(Do),
    "final" => Reserved(Final),
    "else" => Else,
    "elseif" => Elseif,
    "export" => Reserved(Export),
    "fun" => Fun,
    "if" => If,
    "import" => Reserved(Import),
    "in" => Reserved(In),
    "let" => Let,
    "macro" => Reserved(Macro),
    "of" => Reserved(Of),
    "override" => Reserved(Override),
    "private" => Reserved(Private),
    "protected" => Reserved(Protected),
    "pub" => Reserved(Pub),
    "public" => Reserved(Public),
    "pure" => Reserved(Pure),
    "return" => Return,
    "static" => Reserved(Static),
    "struct" => Reserved(Struct),
    "switch" => Reserved(Switch),
    "this" => Reserved(This),
    "trait" => Reserved(Trait),
    "unless" => Unless,
    "use" => Reserved(Use),
    "virtual" => Reserved(Virtual),
    "while" => While,
    "yield" => Reserved(Yield),
};

/// Représente un lexème dans le programme
#[derive(Clone, Debug, Hash, Ord, PartialOrd)]
pub struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) location: PositionOrSpan,
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.token_type == other.token_type
    }
}
impl Eq for Token {}

impl Token {
    pub(crate) fn new(token_type: TokenType, loc: PositionOrSpan) -> Self {
        Token {
            token_type,
            location: loc,
        }
    }

    #[inline]
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    #[inline]
    pub fn location(&self) -> &PositionOrSpan {
        &self.location
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.token_type)
    }
}

macro_rules! token {
    // token!(TokenType::Test, "test", PositionOrSpan::Span(Position::new(), Position::new()))
    ($tokentype:expr, $begin:expr => $end:expr) => {{
        let loc = Span::new($begin, $end);
        token!($tokentype, loc)
    }};

    // token!(TokenType::Test, "test", PositionOrSpan::Position(Position::new(1, 1)))
    ($tokentype:expr, $loc:expr) => {{
        let tok = Token::new($tokentype.into(), $loc.into());
        Ok(tok)
    }};
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TokenType {
    EOF,
    Underscore, // _
    Arrow, // ->

    Eq,             // =
    Plus,           // +
    Minus,          // -
    Division,       // /
    Multiplication, // *
    Modulo,         // %
    Power,          // ^
    Not,            // !

    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    Gt,    // >
    LtEq,  // <=
    GtEq,  // >=

    Or,     // |
    And,    // &
    OrOr,   // ||
    AndAnd, // &&

    Comma,     // ,
    Colon,     // :
    Semicolon, // ;
    Lparen,    // (
    Rparen,    // )
    Lbracket,  // [
    Rbracket,  // ]
    Lbrace,    // {
    Rbrace,    // }

    Illegal(String),
    Identifier(String), // abcdef
    Comment(String),
    Keyword(Keyword),
    Boolean(bool),
    Literal(String),
    Number(Number),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TokenType::*;
        use self::Number::*;
        match self {
            Illegal(st) | Identifier(st) | Comment(st) | Literal(st) => write!(f, "{}", st),
            Keyword(keyword) => write!(f, "{:?}", keyword),
            Number(num) => match num {
                Binary(st) | Octal(st) | Hexadecimal(st) | Decimal(st) => write!(f, "{}", st)
            }
            Boolean(bl) => write!(f, "{}", bl),
            token_type => write!(f, "{:?}", token_type),
        }
    }
}

impl From<Keyword> for TokenType {
    fn from(keyword: Keyword) -> Self {
        TokenType::Keyword(keyword)
    }
}

impl From<Number> for TokenType {
    fn from(number: Number) -> Self {
        TokenType::Number(number)
    }
}

/// Mot-clés du langage
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Keyword {
    Reserved(ReservedKeyword),
    Const,
    Else,
    Elseif,
    Fun,
    If,
    Let,
    Return,
    Unless,
    While,
}

impl Keyword {
    /// Permet de chercher un mot-clé
    pub(crate) fn lookup(keyword: &str) -> Option<Keyword> {
        KEYWORDS.get(keyword).cloned()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReservedKeyword {
    Alias,
    Array,
    Break,
    Case,
    Class,
    Continue,
    Do,
    Export,
    Final,
    Import,
    In,
    Macro,
    Of,
    Override,
    Private,
    Protected,
    Pub,
    Public,
    Pure,
    Static,
    Struct,
    Switch,
    This,
    Trait,
    Use,
    Virtual,
    Yield,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Number {
    Binary(String),
    Decimal(String),
    Hexadecimal(String),
    Octal(String),
}

/// Représente une position dans un programme
/// Peut être employé pour attacher de l'information sur un lexème ou autre
/// IMPORTANT: Position n'est pas relatif à l'entrée, c'est-à-dire
/// que la position ne représente pas un byte précis dans l'entrée
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    /// Ligne
    pub(crate) line: usize,
    /// Colonne
    pub(super) column: usize,
}

impl Position {
    pub(crate) fn new(line: usize, column: usize) -> Self {
        Position {
            line,
            column,
        }
    }

    /// Combine 2 `Position`s en un `Span`
    /// Si le `lhs` est plus petit alors il est la position de début,
    /// sinon si les deux positions sont identiques ou `lhs` est plus grand
    /// que `rhs` alors `Err` est renvoyé.
    pub(crate) fn combine_to_span(lhs: Position, rhs: Position) -> Result<Span, ()> {
        use std::cmp::Ordering::*;
        match lhs.cmp(&rhs) {
            Less => Ok(Span {
                begin: lhs,
                end: rhs,
            }),
            Equal | Greater => Err(()),
        }
    }

    pub fn line(&self) -> usize { self.line }

    pub fn column(&self) -> usize { self.column }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Représente une gamme de caractères
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    pub(crate) begin: Position,
    pub(crate) end: Position,
}

impl Span {
    /// Créé un nouveau `Span` avec les arguments passés en paramètre
    pub(crate) fn new(begin: Position, end: Position) -> Self {
        Span {
            begin,
            end,
        }
    }

    /// Étend une gamme `Span` jusqu'à une position `Position`
    /// Renvoie `Result` indiquant le succès de l'opération.
    /// Renvoie `Err` si la position `rhs` est avant la fin de la gamme.
    pub(crate) fn extend_to(self, rhs: Position) -> Result<Self, ()> {
        use std::cmp::Ordering::*;
        match self.end.cmp(&rhs) {
            Less => Ok(Span {
                begin: self.begin,
                end: rhs,
            }),
            Equal => Ok(self),
            Greater => Err(()),
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.begin, self.end)
    }
}

/// Représente une gamme ou une position dans la séquence de caractères
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PositionOrSpan {
    Position(Position),
    Span(Span),
}

impl convert::From<self::Span> for PositionOrSpan {
    fn from(span: Span) -> Self {
        PositionOrSpan::Span(span)
    }
}

impl convert::From<self::Position> for PositionOrSpan {
    fn from(pos: Position) -> Self {
        PositionOrSpan::Position(pos)
    }
}

impl fmt::Display for PositionOrSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::PositionOrSpan::*;
        match *self {
            Position(ref pos) => fmt::Display::fmt(pos, f),
            Span(ref span) => fmt::Display::fmt(span, f),
        }
    }
}
