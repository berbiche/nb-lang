// TODO(berbiche): Voir comment ce code peut être modélisé pour permettre l'addition de...
// ...nouveaux types d'expression (et autres), une plus grande modularité et extensibilité.
use token::*;

use itertools::Itertools;

use std::fmt;
use std::convert::TryFrom;


/// Représente l'entièreté du programme.
/// Est le noeud racine de l'`ast`.
pub struct Program {
    /// Les énoncés formant le programme
    pub(crate) statements: Vec<Box<Statement>>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            fmt::Display::fmt(stmt, f)?;
        }
        Ok(())
    }
}

#[doc(inline)]
pub(crate) type Identifier = String;

/// Un block est composé de plusieurs énoncés.
/// En dû temps, un `Block` pourra être une expression.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Block(Vec<Statement>);

impl From<Vec<Statement>> for Block {
    fn from(vec: Vec<Statement>) -> Self {
        Block(vec)
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\n")?;
        for stmt in &self.0 {
            fmt::Display::fmt(stmt, f)?;
        }
        write!(f, "}}")
    }
}

/// Une énoncé dans le langage, ces énoncés ne peuvent se retrouver
/// au "top-level", c'est-à-dire qu'une expression n'est pas valide sans son
/// contexte par exemple, tout comme une clause.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Statement {
    /// - 0: La cible de l'affectation
    /// - 1: La valeur qui est assignée,
    Assignment(Variable, Box<Expression>),
    /// Une clause peut se retrouver dans ou en-dehors d'une expression.
    /// La condition est absente pour un else.
    Conditional(Keyword, Option<Box<Expression>>, Block),
    /// Déclaration de fonction
    FunDeclaration(FunctionDeclaration),
    /// Une boucle
    Loop(Keyword, Option<Box<Expression>>, Block),
    /// Une expression
    Expression(Box<Expression>),
    /// La valeur de retour est une `Expression` ou `None`
    Return(Option<Box<Expression>>),
    /// Déclaration de variable
    /// - 0: si c'est un `let`, `const`, etc.
    /// - 1: le sujet de la déclaration
    /// - 2: la valeur assignée
    VariableDeclaration(Keyword, Variable, Box<Expression>),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Statement::*;
        use token::TokenType::*;
        match *self {
            Assignment(ref var, ref exp) => writeln!(f, "{} = {};", var, exp),
            Conditional(ref keyword, ref cond, ref body) => {
                writeln!(f, "")?;
                write!(f, "{token}", token = format!("{:?}", keyword).to_lowercase())?;
                if cond.is_some() {
                    write!(f, " ({condition})\n", condition = cond.as_ref().unwrap())?;
                }
                write!(f, "{{\n{body}\n}}", body = body)
            },
            FunDeclaration(ref fun) => fmt::Display::fmt(fun, f),
            Loop(ref keyword, ref expr, ref body) => {
                writeln!(f, "");
                match *keyword {
                    ::token::Keyword::While => {
                        write!(f, "while(")?;
                        if let Some(ref condition) = expr {
                            write!(f, "{expr}", expr = condition)?;
                        }
                        write!(f, ")")?;
                        writeln!(f, " {{\n{body}\n}}", body = body)
                    },
                    _ => unimplemented!()
                }
            },
            Expression(ref expr) => writeln!(f, "{};", expr),
            Return(ref expr) => match expr {
                Some(ref expr) => writeln!(f, "return {};", expr),
                _ => writeln!(f, "return;"),
            },
            VariableDeclaration(ref keyword, ref ident, ref value) => {
                writeln!(f, "{keyword} {ident} = {value};",
                       keyword = format!("{:?}", keyword).to_lowercase(),
                       ident = ident,
                       value = value,
                )
            },
        }
    }
}

/// Une déclaration de fonction.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FunctionDeclaration {
    /// Le nom de la fonction
    pub identifier: Identifier,
    /// Paramètres de la fonction
    // TODO(berbiche): Devrais-je être réécrit sous la forme suivante?...
    // TODO(berbiche): ...Vec<(identifiant: string, type: string, valeur_par_defaut: Option<Box<Expression>)>
    pub parameters: Vec<Variable>,
    /// Le corps de la fonction
    pub body: Block,
    /// Le type de retour de la fonction
    pub return_type: Type,
}

impl fmt::Display for FunctionDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "fun {id}({params}) -> {return_type} {body}",
               id = self.identifier,
               params = self.parameters.iter().join(", "),
               return_type = self.return_type,
               body = self.body
        )
    }
}

/// Une expression dans le langage
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expression {
    /// Un identifiant consiste seulement en son nom
    Identifier(Identifier),
    /// Tout valeur pouvant être écrite _litérallement_ dans le code
    Literal(Literal),
    /// - 0: l'identifiant de la cible
    /// - 1: les arguments passés à la fonction
    FunCall(Identifier, Vec<Box<Expression>>),
    /// Une expression "binaire" contient un opérateur et deux opérandes
    BinaryExpression(Box<Expression>, BinaryOperator, Box<Expression>),
    /// Une expression "unaire" est une expression où un opérateur
    /// s'applique à une expression, c'est-à-dire que l'expression qui
    /// en résulte n'est pas la concaténation de deux expressions en une.
    /// L'opérateur peut donc être infixe ou suffixe.
    /// L'importance de l'opérateur change l'ordre d'évaluation.
    UnaryExpression(Box<Expression>, UnaryOperator),
}

impl<'a> From<&'a str> for Expression {
    fn from(val: &'a str) -> Self {
        Expression::Identifier(val.to_owned())
    }
}

impl From<Identifier> for Expression {
    fn from(val: Identifier) -> Self {
        Expression::Identifier(val)
    }
}

impl From<Literal> for Expression {
    fn from(val: Literal) -> Self {
        Expression::Literal(val)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Expression::*;
        write!(f, "(")?;
        match *self {
            Identifier(ref st) => write!(f, "{}", st)?,
            Literal(ref lit) => fmt::Display::fmt(lit, f)?,
            FunCall(ref target, ref arguments) => write!(f, "{}({})", target, arguments.iter().join(", "))?,
            BinaryExpression(ref lhs, ref op, ref rhs) => {
                write!(f, "{lhs} {op} {rhs}", lhs = lhs, op = op, rhs = rhs)?;
            },
            UnaryExpression(ref op, ref ex) => write!(f, "")?,
        };
        write!(f, ")")
    }
}

/// Tout ce qui peut être écrit _littéralement_ dans le code:
/// - Nombre,
/// - Chaîne de caractères,
/// - Array,
/// - Booléen
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Literal {
    /// Un tableau unidimensionnel de taille fixe contenant des éléments de même type
    Array(Vec<Box<Expression>>),
    Number(Number),
    String(String),
    Boolean(bool),
}

impl From<Vec<Box<Expression>>> for Literal {
    fn from(val: Vec<Box<Expression>>) -> Self {
        Literal::Array(val)
    }
}

impl From<Number> for Literal {
    fn from(val: Number) -> Self {
        Literal::Number(val)
    }
}

impl<'a> From<&'a str> for Literal {
    fn from(val: &'a str) -> Self {
        Literal::String(val.to_owned())
    }
}

impl From<String> for Literal {
    fn from(val: String) -> Self {
        Literal::String(val)
    }
}

impl From<bool> for Literal {
    fn from(val: bool) -> Self {
        Literal::Boolean(val)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Literal::*;
        use ::std::fmt::{Display, Debug};
        match *self {
            Array(ref arr) => Debug::fmt(arr, f),
            Number(ref num) => Display::fmt(num, f),
            String(ref st) => write!(f, "{}", st),
            Boolean(ref bl) => Debug::fmt(bl, f),
        }
    }
}


/// Un nombre dans le langage
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Number {
    /// Un nombre décimal à double précision
    Float(f64),
    /// Un nombre entier signé 32 bits (-2^16 à 2^16-1)
    Int(i32),
    /// Un nombre entier signé 64 bits (-2^32 à 2^32-1)
    Long(i64),
}

impl From<f64> for Number {
    fn from(val: f64) -> Self {
        Number::Float(val)
    }
}

impl From<i32> for Number {
    fn from(val: i32) -> Self {
        Number::Int(val)
    }
}

impl From<i64> for Number {
    fn from(val: i64) -> Self {
        Number::Long(val)
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Number::*;
        match *self {
            Long(num) => write!(f, "{}", num),
            Float(num) => write!(f, "{}", num),
            Int(num) => write!(f, "{}", num),
        }
    }
}

/// Tout opérateur pouvant se retrouver en position infixe dans une expression.
/// Ces opérateurs peuvent uniquement se retrouver dans une expression "binaire".
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BinaryOperator {
    Div,
    EqEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Min,
    Mod,
    Mul,
    NE,
    Plus,
    Pow
}

impl TryFrom<TokenType> for BinaryOperator {
    type Error = ();
    fn try_from(token_type: TokenType) -> Result<Self, Self::Error> {
        use token::TokenType as tt;
        use self::BinaryOperator as bo;

        Ok(match token_type {
            tt::Division => bo::Div,
            tt::EqEq => bo::EqEq,
            tt::Gt => bo::Gt,
            tt::GtEq => bo::GtEq,
            tt::Lt => bo::Lt,
            tt::LtEq => bo::LtEq,
            tt::Minus => bo::Min,
            tt::Modulo => bo::Mod,
            tt::Multiplication => bo::Mul,
            tt::NotEq => bo::NE,
            tt::Plus => bo::Plus,
            tt::Power => bo::Pow,
            _ => return Err(())
        })
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BinaryOperator::*;
        write!(f, "{}", match self {
            Div => "/",
            EqEq => "==",
            Gt => ">",
            GtEq => ">=",
            Lt => "<",
            LtEq => "<=",
            Min => "-",
            Mod => "%",
            Mul => "*",
            NE => "!=",
            Plus => "+",
            Pow => "^",
        })
    }
}

/// Tout opérateur s'appliquant à un opérande
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UnaryOperator {
    Not,
}

impl TryFrom<TokenType> for UnaryOperator {
    type Error = ();
    fn try_from(token_type: TokenType) -> Result<Self, Self::Error> {
        use token::TokenType as tt;
        use self::UnaryOperator as uo;

        Ok(match token_type {
            tt::Not => uo::Not,
            _ => return Err(())
        })
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UnaryOperator::*;
        write!(f, "{}", match *self {
            Not => '!',
        })
    }
}

/// Une variable dans le langage
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Variable {
    /// Nom de l'identifiant
    pub name: Identifier,
    /// Le type de variable (type est un mot réservé dans Rust)
    pub category: Type
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.category)
    }
}

/// Unité contenant l'information sur un type
/// Pour l'instant, cette unité va se limiter à une chaîne de caractères
/// contenant uniquement le nom du type.
// TODO: Me déplacer dans mon propre module
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Type {
    /// Nom du type
    pub name: String,
//    /// Visibilité du type
//    visibility:
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// Quelques tests pour voir si le formattage fonctionne correctement
#[cfg(test)]
mod test {
    use super::{
        *,
        Statement::*,
        TokenType::*,
        Keyword,
        Expression,
        Literal,
    };
    use ast;

    #[test]
    fn variable_declaration() {
        let expected = "let value: int = ((5) + (10));\n";
        let va = Statement::VariableDeclaration(
            Keyword::Let,
            Variable { name: "value".to_string(), category: Type {name: "int".to_string()} },
            box Expression::BinaryExpression(
                box Expression::Literal(Literal::Number(ast::Number::Int(5))),
                BinaryOperator::Plus,
                box Expression::Literal(Literal::Number(ast::Number::Int(10))),
            )
        );

        assert_eq!(expected, format!("{}", va));
    }

    #[test]
    fn function_declaration() {
        let expected = "\
fun Allo(p1: int, p2: string) -> string {
let a: string = (1);
return ((a) + (2));
}
";
        let va = FunctionDeclaration {
            identifier: "Allo".to_string(),
            parameters: vec![
                Variable {
                    name: "p1".to_string(),
                    category: Type { name: "int".to_string() }
                },
                Variable {
                    name: "p2".to_string(),
                    category: Type { name: "string".to_string() }
                },
            ],
            body: Block(vec![
                VariableDeclaration(
                    Keyword::Let,
                    Variable {
                        name: "a".to_string(),
                        category: Type {
                            name: "string".to_string(),
                        },
                    },
                    box Literal::Number(1.into()).into()
                ).into(),
                Statement::Return(
                    Some(box Expression::BinaryExpression(
                        box Literal::String("a".to_string()).into(),
                        BinaryOperator::Plus,
                        box Literal::Number(2.into()).into(),
                    )),
                ),
            ]),
            return_type: Type {
                name: "string".to_string(),
            },
        };

        assert_eq!(expected, format!("{}", va));
    }
}
