use ast::{self, Program};
use lexer::{self, Lexer, error::{Error, LResult}};
use token::{self, Token, TokenType, Number, Keyword};

use std::collections::{HashMap, HashSet};
use std::mem;

//use phf;


/// Collection contenant les identifiants de ce `Parser`
//type Map<'a, 'b: 'a> = &'a StdHashMap<&'b str, token::Token>;
pub type Map<'a, 'b> = HashMap<&'b str, &'b str>;

/// Renvoie l'importance du token actuel
/// Ordre de précédence des opérateurs
fn get_precedence(token: &Token) -> u8 {
    use token::TokenType::*;
    match token.token_type {
        EqEq => 5,
        OrOr => 5,
        AndAnd => 5,
        Not => 10,
        Plus => 20,
        Minus => 20,
        Division => 25,
        Multiplication => 25,
        Modulo => 25,
        Power => 30,
        Lparen => 0,
        Rparen => 0,
        Eq => 255,
        _ => 5,
    }
}

/// Le `Parser` contient toutes les informations associés au "parsage" d'une entrée
#[derive(Debug)]
pub struct Parser<'a, 'b: 'a> {
    /// L'instance du Lexer a utilisé
    lexer: Lexer<'a>,
    /// Le `Token` sur lequel le `Parser` est présentement
    cur_token: Option<Token>,
    /// Le prochain `Token`
    peek_token: Option<Token>,
    /// Structure de données contenant les identifiants rencontrés
    identifiers: Map<'a, 'b>,
    /// Les erreurs rencontrés par le `Parser`
    errors: Vec<Error>,
}

impl<'a, 'b: 'a> Parser<'a, 'b> {
    /// Crée une nouvelle instance de `Parser` qui utilise le `Lexer` et le `Map`
    /// passés en arguments
    fn new(lexer: Lexer<'a>, identifiers: Map<'a, 'b>) -> Self {
        Parser {
            lexer,
            identifiers,
            cur_token: None,
            peek_token: None,
            errors: Vec::new(),
        }
    }

    /// Crée une nouvelle de `Parser` qui utilise le `Lexer` passé en argument
    #[inline]
    pub fn from_lexer(lexer: Lexer<'a>) -> Self {
        Parser::new(lexer, HashMap::new())
    }

    /// Crée une nouvelle de `Parser` qui utilise l'entrée (chaîne de caractères)
    /// passé en argument
    #[inline]
    pub fn from_source(input: &'a str) -> Self {
        Parser::from_lexer(Lexer::new(input))
    }

    /// Permet l'ajout d'une map d'identifiant au `Parser`
    pub fn with_identifiers(&mut self, idents: Map<'a, 'b>) {
        self.identifiers = idents;
    }

    /// Consomme le `Lexer` et le `Parser` et renvoie
    /// le AST du programme ou un vecteur de `Error`
    pub fn parse(mut self) -> Result<ast::Program, Vec<Error>> {
        // avance le parser pour que peek_token soit set
        self.advance_token();
        // avance le parser a nouveau pour que cur_token soit set
        self.advance_token();

        // l'instance du programme que nous allons retourner
        let mut program = Program::new();
        loop {
            match self.cur_token {
                Some(..) => match self.parse_statement() {
                    Ok(stmt) => program.statements.push(box stmt),
                    Err(error) => self.errors.push(error),
                },
                None => break,
            };
            self.advance_token();
        }

        if self.errors.is_empty() {
            Ok(program)
        }
        else {
            Err(self.errors)
        }
    }

    /// Lit le prochain `Token` dans `peek_token` et avance le `cur_token`
    /// au `Token` de `peek_token`
    /// Nécessite d'être invoquer 2 fois initialement car
    fn advance_token(&mut self) {
        // remplace la valeur dans cur_token pour celle de peek_token
        // et ajuste la valeur de peek_token à None
        self.cur_token = mem::replace(&mut self.peek_token, None);

        // avance le peek_token au prochain token
        self.peek_token = match self.lexer.read_token() {
            Ok(token) => match token.token_type {
                TokenType::EOF => None,
                TokenType::Illegal(st) => {
                    let error = Error::UnexpectedCharacter(st, token.location);
                    self.errors.push(error);
                    None
                },
                _ => Some(token),
            },
            Err(error) => {
                self.errors.push(error);
                None
            },
        };
    }

    /// Renvoie le `cur_token`, le remplaçant par la valeur de `peek_token`
    #[inline]
    fn cur_token(&mut self) -> Option<Token> {
        let result = mem::replace(&mut self.cur_token, None);
        self.advance_token();
        result
    }

    /// Renvoie la précédence du `Token` actuel
    #[inline]
    fn cur_precedence(&self) -> u8 {
        self.cur_token.as_ref().map_or(0, get_precedence)
    }

    /// Renvoie la précédence du prochain `Token`
    #[inline]
    fn peek_precedence(&self) -> u8 {
        self.peek_token.as_ref().map_or(0, get_precedence)
    }

    /// Renvoie si le `cur_token` est le `TokenType` passé en argument
    #[inline]
    fn cur_token_is(&self, kind: &TokenType) -> bool {
        match self.cur_token {
            Some(ref token) => compare_token_kind(&token.token_type, kind),
            _ => false
        }
    }

    /// Renvoie si le `peek_token` est le `TokenType` passé en argument
    #[inline]
    fn peek_token_is(&self, kind: &TokenType) -> bool {
        match self.peek_token {
            Some(ref token) => compare_token_kind(&token.token_type, kind),
            _ => false
        }
    }

    /// Renvoie une erreur `Error::ExpectedToken` si le `cur_token`
    /// n'est pas celui désiré
    fn expect_token(&mut self, kind: &TokenType) -> LResult<()> {
        match self.cur_token {
            Some(ref token) if compare_token_kind(&token.token_type, kind) => Ok(()),
            _ => Err(Error::UnexpectedEOF(self.lexer.position().into()))
        }
    }

    /// Parse un énoncé, quel qu'il soit et le renvoie
    /// Si quoique se soit est illégal dans l'énoncé, une erreur est générée
    fn parse_statement(&mut self) -> LResult<ast::Statement> {
        use token::{
            TokenType::{self, *},
            Keyword::{self, *},
        };
        match self.cur_token.as_ref().unwrap().token_type {
            Keyword(Keyword::Let) | Keyword(Keyword::Const) => self.parse_variable_declaration(),
            Keyword(Keyword::Fun) => self.parse_function_declaration(),
            Keyword(Keyword::Return) => self.parse_return(),
            Keyword(Keyword::If) => self.parse_if_statement(),
            Keyword(Keyword::Unless) => self.parse_unless_statement(),
            Keyword(Keyword::While) => self.parse_while_loop(),
            Keyword(Keyword::Reserved(_)) => error_reserved_keyword(self.cur_token().unwrap()),
            _ => self.parse_statement_expression(),
        }
    }

    /*
        Section contenant le code pour "parser" les `Token`s
        en un "node" `ast`.
    */
    /// Parse une déclaration de variable
    fn parse_variable_declaration(&mut self) -> LResult<ast::Statement> {
        let declaration_keyword = self.cur_token().unwrap().token_type;

        self.expect_token(&TokenType::Identifier("".to_string()))?;
        let ident = self.parse_identifier()?;

//        Ok(ast::Statement::VariableDeclaration(declaration_keyword, ))
        unimplemented!()
    }

    /// Parse une déclaration de fonction
    fn parse_function_declaration(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    fn parse_statement_block(&mut self) -> LResult<ast::Block> {
        unimplemented!()
    }

    /// Parse un énoncé-expression
    fn parse_statement_expression(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse un retour de fonction
    fn parse_return(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une expression conditionnelle `if`
    fn parse_if_statement(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une expression conditionnelle `unless`
    fn parse_unless_statement(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    fn parse_while_loop(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une expression entre parenthèses
    fn parse_paren_expression(&mut self) -> LResult<Box<ast::Expression>> {
        unimplemented!()
    }

    /// Parse une expression binaire
    /// Plus d'information sur ce qu'est une expression binaire...
    /// dans `ast::BinaryOperator`
    fn parse_binary_expression(&mut self) -> LResult<Box<ast::Expression>> {
        match self.cur_token() {
            Some(token) => {
                unimplemented!()
            },
            None => Err(Error::UnexpectedEOF(self.lexer.position().into())),
        }
    }

    fn parse_call_expression(&mut self) -> LResult<Box<ast::Expression>> {
        unimplemented!()
    }

    /// Parse une liste d'expression, c'est-à-dire une liste d'éléments
    /// séparés par des virgules
    /// - Terminator: Le token qui termine la séquence
    fn parse_expression_list(&mut self, terminator: token::TokenType)
        -> LResult<Vec<Box<ast::Expression>>> {
//        let vec = vec![];
//        match self.cur_token {
//            Some(ref t) if self.cur_token_is(t) => Ok(vec),
//        }
        unimplemented!()
    }

    /// Parse un literal (nombre, booléen, array, string)
    /// Nombre: faute d'avoir une meilleure solution, les nombres sont représentés
    /// par un f64 durant cette phase de compilation
    // TODO(berbiche): M'extraire en des fonctions (parse_number, parse_array, etc.)
    fn parse_literal(&mut self) -> LResult<ast::Literal> {
        match self.cur_token.as_ref().unwrap().token_type {
            TokenType::Boolean(_) => self.parse_boolean(),
            TokenType::Literal(_) => self.parse_string(),
            TokenType::Number(_) => self.parse_number().map(ast::Literal::from),
            TokenType::Lbracket => self.parse_array(),
            _ => error_unexpected_token(self.cur_token().unwrap())
        }
    }

    /// Parse un array
    fn parse_array(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Lbracket =>
                self.parse_expression_list(TokenType::Rbracket).map(ast::Literal::from),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un booléen
    fn parse_boolean(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Boolean(b) => Ok(ast::Literal::Boolean(b)),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un nombre
    fn parse_number(&mut self) -> LResult<ast::Number> {
        let token = self.cur_token().unwrap();

        // parse un numéro dans la base donnée
        // (String -> i32|i64 -> ast::Number)
        // Présentement, il n'est pas possible de déterminer la cause d'erreur
        // ... car les enum std::num::IntErrorKind et std::num::FloatErrorKind sont privés
        // ... `parse`/`from_str_radix` renvoie
        // ... `ParseIntError { kind: IntErrorKind/FloatErrorKind }`
        fn parse_with_base(num: String, base: u32, location: token::PositionOrSpan)
                           -> LResult<ast::Number> {
            let num = num.replace("_", "");

            // e
            let mut number = i32::from_str_radix(num.as_ref(), base).map(ast::Number::from);
            if number.is_err() {
                number = i64::from_str_radix(num.as_ref(), base).map(ast::Number::from);
            }
            number.map_err(|_err| Error::InvalidNumber(num, location))
        }

        match token.token_type {
            TokenType::Number(number) => match number {
                Number::Binary(num) =>  parse_with_base(num, 2, token.location),
                Number::Octal(num) => parse_with_base(num, 8, token.location),
                Number::Hexadecimal(num) => parse_with_base(num, 16, token.location),
                Number::Decimal(num) => {
                    let num = num.replace("_", "");
                    // Converti nombre -> ast::Number
                    let success = num.parse::<i32>().map(ast::Number::from)
                        .or_else(|_| num.parse::<i64>().map(ast::Number::from))
                        .or_else(|_| num.parse::<f64>().map(ast::Number::from));
                    // ne compile pas, problème avec burrowck
//                    .map_err(|_| Error::InvalidNumber(num, token.location));
                    // alternative
                    match success {
                        Err(_) => Err(Error::InvalidNumber(num, token.location)),
                        Ok(t) => Ok(t)
                    }
                },
            },
            _ => error_unexpected_token(token)
        }
    }

    /// Parse une chaîne de caractères
    fn parse_string(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Literal(st) => Ok(ast::Literal::String(st)),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un identifiant, une appelle de fonction ou l'indexage d'un Array
    // FIXME(berbiche): Bouger la logique pour parser autre qu'un identifiant en-dehors
    fn parse_identifier(&mut self) -> LResult<Box<ast::Expression>> {
        let token = self.cur_token().unwrap();
        // cur_token est désormais peek_token
        match self.cur_token.as_ref() {
            // appelle de fonction
            Some(Token { token_type: TokenType::Lparen, .. }) => {
                self.advance_token(); // consomme peek_token
                self.parse_expression_list(TokenType::Rparen)
                    .and_then(|args| match token {
                        Token { token_type: TokenType::Identifier(st), .. } => {
                            Ok(box ast::Expression::FunCall(st, args))
                        },
                        _ => error_unexpected_token(token)
                    })
            },
            Some(Token { token_type: TokenType::Lbracket, .. }) => {
                self.advance_token();
                unimplemented!()
            }
            // ok juste un identifiant
            _ => match token.token_type {
                TokenType::Identifier(st) => Ok(box ast::Expression::Identifier(st)),
                _ => error_unexpected_token(token),
            }
        }
    }
}

/*
    Section de code pour les helpers
*/
/// Compare le variant de deux TokenType et renvoie s'ils sont le même
#[inline]
fn compare_token_kind(lhs: &TokenType, rhs: &TokenType) -> bool {
    mem::discriminant(lhs) == mem::discriminant(rhs)
}

/*
    Section contenant le code pour créer, gérer et modifier des `LResult`
*/
/// Renvoie une erreur de mot-clé réservé
#[inline]
fn error_reserved_keyword<T>(token: Token) -> LResult<T> {
    Err(Error::ReservedKeyword(token.token_type, token.location))
}

/// Renvoie une erreur de `Token` inattendu
#[inline]
fn error_unexpected_token<T>(token: Token) -> LResult<T> {
    Err(Error::UnexpectedToken(token.token_type, token.location))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test() {

    }
}
