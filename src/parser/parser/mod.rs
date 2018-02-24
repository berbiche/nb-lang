use ast::{self, Program};
use lexer::{Lexer, error::{Error, LResult}};
use token::{Token, TokenType};

use std::collections::{HashMap, HashSet};
use std::mem;

//use phf;


/// Collection contenant les identifiants de ce `Parser`
//type Map<'a, 'b: 'a> = &'a StdHashMap<&'b str, token::Token>;
pub type Map<'a: 'b, 'b> = HashMap<&'b str, &'b str>;

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
//        self.peek_token = self.lexer.read_token()
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

    /// Parse un énoncé, quel qu'il soit et le renvoie
    /// Si quoique se soit est illégal dans l'énoncé, une erreur est générée
    fn parse_statement(&mut self) -> LResult<ast::Statement> {
        use token::{
            TokenType::{self, *},
            Keyword::{self, *},
        };
        match self.cur_token.unwrap().token_type {
            Keyword(Keyword::Let) | Keyword(Keyword::Const) => self.parse_variable_declaration(),
            Keyword(Keyword::Fun) => self.parse_function_declaration(),
            Keyword(Keyword::Return) => self.parse_return(),
            Keyword(Keyword::If) => self.parse_if_expression(),
            Keyword(Keyword::Unless) => self.parse_unless_expression(),
            Keyword(Keyword::While) => self.parse_while_loop(),
            _ => self.parse_statement_expression(),
        }
    }

    /*
        Section contenant le code pour "parser" les `Token`s
        en un "node" `ast`.
    */
    fn parse_variable_declaration(&mut self) -> LResult<ast::Statement> {
        let token = self.cur_token.unwrap();
        match token.token_type {

        }
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
    fn parse_if_expression(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une expression conditionnelle `unless`
    fn parse_unless_expression(&mut self) -> LResult<ast::Statement> {
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
        match self.cur_token {
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
    fn parse_expression_list(&mut self) -> LResult<Vec<Box<ast::Expression>>> {
        unimplemented!()
    }

    /// Parse un literal
    fn parse_literal(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token.unwrap();
        match token.token_type {
            TokenType::Boolean(b) => Ok(ast::Literal::Boolean(b)),
            _ => Err(Error::UnexpectedToken(token, token.location))
        }
    }

    /// Parse un identifiant ou une appelle de fonction
    fn parse_identifier(&mut self) -> LResult<Box<ast::Expression>> {
        let token = self.cur_token.unwrap();
        let lookahead = self.peek_token;
        match lookahead {
            Some(TokenType::Lparen) => { // appelle de fonction
                self.advance_token(); // consomme le token
                let args = self.parse_expression_list()?;

                if let Token { token_type: TokenType::Identifier(st), .. } = token {
                    Ok(ast::Expression::FunCall(st, args))
                }
                else {
                    Err(Error::UnexpectedToken(token, token.location))
                }
            },
            _ => match token.token_type {
                TokenType::Identifier(st) => Ok(ast::Expression::Identifier(st)),
                _ => Err(Error::UnexpectedToken(token, token.location)),
            }
        }
    }

//    fn parse_identifier(&mut self) -> ast::I
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test() {

    }
}
