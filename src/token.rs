#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    EOF, //
    Identifier, // abcdef
    Underscore, // _

    Assign, // =
    Plus, // +
    Minus, // -
    Division, // /
    Modulo, // %
    Power, // ^
    Not, // !

    EqEq, // ==
    NotEq, // !=
    Lt, // <
    Gt, // >
    LtEq, // <=
    GtEq, // >=

    OrOr, // ||
    AndAnd, // &&

    Comma, // ,
    Semicolon, // ;
    Lparen, // (
    Rparen, // )
    Lbracket, // {
    Rbracket, // }
    Lbrace, // [
    Rbrace, // ]

    Comment(Comment),
    Keyword(Keywords),
    Boolean(Boolean)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Comment {
    Line, // //
    Block(BlockComment)
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockComment {
    Start, // /*
    End // */
}

/// Includes reserved keywords
#[derive(PartialEq, Debug, Clone)]
pub enum Keywords {
    Alias,
    Array,
    Break,
    Case,
    Class,
    Const,
    Continue,
    Do,
    Else,
    Elseif,
    Export,
    Fun,
    If,
    Import,
    In,
    Let,
    Of,
    Private,
    Protected,
    Pub,
    Public,
    Return,
    Static,
    Struct,
    Switch,
    Unless,
    Use,
    While
}

#[derive(PartialEq, Debug, Clone)]
pub enum Boolean {
    TRUE,
    FALSE
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    literal: String
}
