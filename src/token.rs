use phf;

use std::convert;
use std::fmt;

use self::{Keyword::*, ReservedKeyword::*};

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "alias" => Reserved(Alias),
    "array" => Reserved(Array),
    "break" => Break,
    "case" => Reserved(Case),
    "class" => Reserved(Class),
    "const" => Const,
    "continue" => Continue,
    "do" => Do,
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
    "struct" => Struct,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    token_type: TokenType,
    location: PositionOrSpan,
}

impl Token {
    pub(crate) fn new(token_type: TokenType, loc: PositionOrSpan) -> Self {
        Token {
            token_type,
            location: loc,
        }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenType {
    EOF,
    Underscore, // _

    Eq,       // =
    Plus,     // +
    Minus,    // -
    Division, // /
    Modulo,   // %
    Power,    // ^
    Not,      // !

    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    Gt,    // >
    LtEq,  // <=
    GtEq,  // >=

    Or, // |
    And, // &
    OrOr,   // ||
    AndAnd, // &&

    Comma,     // ,
    Semicolon, // ;
    Lparen,    // (
    Rparen,    // )
    Lbracket,  // {
    Rbracket,  // }
    Lbrace,    // [
    Rbrace,    // ]

    Illegal(String),
    Identifier(String), // abcdef
    Comment(String),
    Keyword(Keyword),
    Boolean(Boolean),
    Literal(String),
    Number(Number),
}

impl From<Keyword> for TokenType {
    fn from(keyword: Keyword) -> Self {
        TokenType::Keyword(keyword)
    }
}

impl From<Boolean> for TokenType {
    fn from(boolean: Boolean) -> Self {
        TokenType::Boolean(boolean)
    }
}

impl From<Number> for TokenType {
    fn from(number: Number) -> Self {
        TokenType::Number(number)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    Reserved(ReservedKeyword),
    Break,
    Const,
    Continue,
    Do,
    Else,
    Elseif,
    Fun,
    If,
    Let,
    Return,
    Struct,
    Unless,
    While,
}

impl Keyword {
    /// Permet de chercher un mot-clé
    pub(crate) fn lookup(keyword: &str) -> Option<Keyword> {
        KEYWORDS.get(keyword).cloned()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservedKeyword {
    Alias,
    Array,
    Case,
    Class,
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
    Switch,
    This,
    Trait,
    Use,
    Virtual,
    Yield,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Boolean {
    True,
    False,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
    begin: Position,
    end: Position,
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
    pub(crate) fn extend_to(&mut self, rhs: Position) -> Result<(), ()> {
        use std::cmp::Ordering::*;
        match self.end.cmp(&rhs) {
            Less => {
                self.end = rhs;
                Ok(())
            },
            Equal => Ok(()),
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
