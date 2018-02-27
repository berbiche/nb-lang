//! Code pour l'implémentation du parser.
//!
//! Le parser implémenté est un _recursive descent parser_, couplé à un _Pratt parser_
//! pour le "parsage" des expressions.
//!
//! Présentement, le parser n'a pas "d'error recovery", c'est-à-dire que lorsqu'un token
//! illégal est rencontré une erreur de type `lexer::error::Error` est généré.
//! J'ai pu trouver deux stratégies, l'une est de sauter les tokens jusqu'à avoir trouvé
//! un token "safe", l'autre étant d'avoir un système de correction d'erreur, où l'on va
//! insérer et retirer des tokens jusqu'à avoir une syntaxe valide.
//! Un token "safe" est un token qui, pour l'expression, l'énoncé, ou autre,
//! jusqu'auquel on doit consommer l'input pour finir l'évaluation du noeud de l'AST.
//! Ce token pourrait être le semicolon ';', la paranthèse fermante ')', etc.
//!
//! L'entièreté des fonctions sont écrites sous l'impression que l'appellant
//! aura fait les vérifications préalables avant d'appeler une fonction de parse
//! spécifique, c'est-à-dire que si deux syntaxes peuvent mener à différentes choses,
//! alors l'appelant aura vérifiquer qu'il invoque la bonne syntaxe (d'où le 1 token
//! de look-ahead).
//! De ce fait, la tâche de reprendre d'une erreur est mise sur l'appelant.
//!
//! Par exemple, pour déterminer s'il y a présence d'une appel de fonction ou de l'indexage
//! d'un array, il faut déterminer si le `peek_token` est `TokenType::Lparen` ou
//! `TokenType::Rbracket`, dans le cas contraire nous avons simplement un identifiant.
use ast::{self, Program};
use lexer::{self, Lexer, error::{Error, LResult}};
use token::{self, Token, TokenType, Number, Keyword};

use std::collections::{HashMap, HashSet};
use std::mem;


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

/// Compare le variant de deux TokenType, renvoie le résultat
#[inline]
fn is_same_tokentype(lhs: &TokenType, rhs: &TokenType) -> bool {
    mem::discriminant(lhs) == mem::discriminant(rhs)
}

/// Renvoie une erreur de mot-clé réservé
#[inline]
fn error_reserved_keyword<T>(token: Token) -> LResult<T> {
    Err(Error::ReservedKeyword(token.token_type, token.location))
}

/// Crée une erreur de `Token` inattendu `Error::UnexpectedToken`
///
/// La plupart des invocations de cette fonction pour être changé pour l'appel
/// de la fonction LLVM intrinsèque `::std::intrinsics::unreachable()`,
/// cette dernière illustre un chemin qui ne devrait **JAMAIS** être atteint.
/// Comme expliqué plus haut, dû au système de typage et la manière dont le parser
/// a été écrit, on connait le type de token lorsqu'une fonction précise se fait appeler,
/// mais il n'est toutefois pas possible de démontrer cela avec les `enum` de _Rust_
/// (au meilleur de ma connaissance (incluant les méthodes intrinsèques/unsafe))
#[inline]
fn error_unexpected_token<T>(token: Token) -> LResult<T> {
    Err(Error::UnexpectedToken(token.token_type, token.location))
}

/// Crée une erreur `Error::UnexpectedEOF`
#[inline]
fn error_unexpected_eof<P, T>(pos: P) -> LResult<T> where P: Into<token::PositionOrSpan> {
    Err(Error::UnexpectedEOF(pos.into()))
}

fn error_expected_token<S, T>(st: S, token: Token) -> LResult<T> where S: Into<String> {
    Err(Error::ExpectedToken(st.into(), token.token_type, token.location))
}


/// Le `Parser` contient toutes les informations associés au "parsage" d'une entrée.
/// La table des symboles sera construite au prochain stage, où l'on marche l'AST.
#[derive(Debug)]
pub struct Parser<'a> {
    /// L'instance du Lexer a utilisé
    lexer: Lexer<'a>,
    /// Le `Token` sur lequel le `Parser` est présentement
    cur_token: Option<Token>,
    /// Le prochain `Token`
    peek_token: Option<Token>,
    /// Les erreurs rencontrés par le `Parser`.
    /// L'idée est de permettre au `Parser` d'essayer de recouvrir et continuer à parser
    /// même lorsqu'une erreur est rencontré.
    errors: Vec<Error>,
}

// TODO(berbiche): Ajouter fonction pour contraintes génériques et l'`ast` pour
impl<'a> Parser<'a> {
    /// Crée une nouvelle instance de `Parser` qui utilise le `Lexer` et le `Map`
    /// passés en arguments.
    fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            lexer,
            cur_token: None,
            peek_token: None,
            errors: Vec::new(),
        }
    }

    /// Crée une nouvelle de `Parser` qui utilise l'entrée (chaîne de caractères)
    /// passé en argument.
    #[inline]
    pub fn from_source(input: &'a str) -> Self {
        Parser::new(Lexer::new(input))
    }

    /// Consomme le `Lexer` et le `Parser` et renvoie le AST du programme ou un
    /// vecteur de `Error`.
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
    /// au `Token` de `peek_token`.
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
            Some(ref token) => is_same_tokentype(&token.token_type, kind),
            _ => false
        }
    }

    /// Renvoie si le `peek_token` est le `TokenType` passé en argument
    #[inline]
    fn peek_token_is(&self, kind: &TokenType) -> bool {
        match self.peek_token {
            Some(ref token) => is_same_tokentype(&token.token_type, kind),
            _ => false
        }
    }

    /// Renvoie une erreur `Error::ExpectedToken` si le `cur_token`
    /// n'est pas celui désiré
    fn expect_token(&self, kind: &TokenType) -> LResult<()> {
        match self.cur_token {
            Some(ref token) if is_same_tokentype(&token.token_type, kind) => Ok(()),
            _ => Err(Error::UnexpectedEOF(self.lexer.position().into()))
        }
    }

    /// Renvoie une erreur `Error::ExpectedToken` si le `cur_token`
    /// n'est pas un identifiant
    #[inline]
    fn expect_ident(&self) -> LResult<()> {
        self.expect_token(&TokenType::Identifier(String::new()))
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
            _ => self.parse_expression_statement(),
        }
    }

    /*
        Section contenant le code pour "parser" les `Token`s
        en un "node" `ast`.
    */
    /// Parse une déclaration de variable
    fn parse_variable_declaration(&mut self) -> LResult<ast::Statement> {
        use ast::Statement::VariableDeclaration;

        let declaration_keyword = {
            let token = self.cur_token().unwrap();
            match token.token_type {
                TokenType::Keyword(Keyword::Let) => Keyword::Let,
                TokenType::Keyword(Keyword::Const) => Keyword::Const,
                _ => return error_unexpected_token(token)
            }
        };

        self.expect_ident()?;
        let variable = {
            let ident = self.parse_identifier()?;
            ast::Variable {
                name: ident,
                category: if self.cur_token_is(&TokenType::Colon) {
                    self.parse_type()?
                }
                else {
                    ast::Type { name: String::new() }
                },
            }
        };

        if self.cur_token.is_none() {
            error_unexpected_eof(self.lexer.position())?
        }

        let value = self.parse_expression()?;
        Ok(VariableDeclaration(declaration_keyword, variable, value))
    }

    /// Parse une déclaration de fonction.
    /// Aucun support pour les fonctions génériques.
    fn parse_function_declaration(&mut self) -> LResult<ast::Statement> {
        use self::TokenType::Identifier;
        self.advance_token(); // consomme le `fun`

        let identifier = match self.cur_token.as_ref() {
            Some(t) if is_same_tokentype(&t.token_type, &Identifier(String::new())) =>
                self.parse_identifier()?,
            _ => error_unexpected_eof(self.lexer.position())?
        };

        self.expect_token(&TokenType::Lparen)?;
        self.advance_token();

        let parameters = self.parse_function_prototype()?;

        // parse le return type de la fonction
        let return_type = match self.cur_token.as_ref() {
            Some(Token { token_type: TokenType::Arrow, .. }) => {
                    self.advance_token();
                    self.expect_ident()?;
                    let typ = self.parse_identifier()?;
                    ast::Type { name: typ.to_string() }
            },
            Some(_) => {
                self.advance_token();
                ast::Type { name: String::new() }
            },
            None => error_unexpected_eof(self.lexer.position())?
        };

        self.expect_token(&TokenType::Rbrace)?;
        let body = self.parse_statement_block()?;

        Ok(ast::Statement::FunDeclaration(ast::FunctionDeclaration {
            identifier,
            body,
            parameters,
            return_type,
        }))
    }

    /// Parse le prototype (signature) d'une fonction.
    /// Pourra même fonctionner avec les lambda si jamais ceux-ci sont ajoutés.
    fn parse_function_prototype(&mut self) -> LResult<Vec<ast::Variable>> {
        let mut prototype = vec![];
        while !self.cur_token_is(&TokenType::Rparen) {
            let identifier = match self.cur_token.as_ref() {
                Some(Token { token_type: TokenType::Identifier(_), .. }) =>
                    self.parse_identifier()?,
                Some(_) => { // pas un identifiant
                    let token = self.cur_token().unwrap();
                    error_expected_token("identifier", token)?
                },
                _ => error_unexpected_eof(self.lexer.position())?
            };

            self.expect_token(&TokenType::Colon)?;
            self.advance_token();

            let typ = match self.cur_token() {
                Some(token) => match token.token_type {
                    TokenType::Identifier(st) => self.parse_type()?,
                    _ => error_expected_token("type", token)?
                },
                _ => error_unexpected_eof(self.lexer.position())?
            };

            prototype.push(ast::Variable {
                name: identifier,
                category: typ,
            });
        }

        Ok(prototype)
    }

    /// Parse un bloc d'énoncés.
    fn parse_statement_block(&mut self) -> LResult<ast::Block> {
        if self.cur_token_is(&TokenType::Lbrace) {
            self.advance_token();
        }

        let mut block = vec![];
        while !self.cur_token_is(&TokenType::Rbrace) {
            block.push(self.parse_statement()?);
        }

        Ok(block.into())
    }

    /// Parse un énoncé-expression.
    /// Une expression seule est un énoncé valide dans le langage.
    fn parse_expression_statement(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse un retour de fonction.
    fn parse_return(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une énoncé conditionnelle `if`.
    fn parse_if_statement(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une énoncé conditionnelle `unless`.
    fn parse_unless_statement(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une boucle `while`.
    fn parse_while_loop(&mut self) -> LResult<ast::Statement> {
        unimplemented!()
    }

    /// Parse une expression.
    fn parse_expression(&mut self) -> LResult<Box<ast::Expression>> {
        unimplemented!()
    }

    /// Parse une expression entre parenthèses.
    fn parse_paren_expression(&mut self) -> LResult<Box<ast::Expression>> {
        unimplemented!()
    }

    /// Parse une expression binaire.
    /// Plus d'information sur ce qu'est une expression binaire dans `ast::BinaryOperator`.
    fn parse_binary_expression(&mut self) -> LResult<Box<ast::Expression>> {
        match self.cur_token() {
            Some(token) => {
                unimplemented!()
            },
            None => Err(Error::UnexpectedEOF(self.lexer.position().into())),
        }
    }

    /// Parse un appel de fonction.
    fn parse_call_expression(&mut self) -> LResult<Box<ast::Expression>> {
        let ident = self.parse_identifier()?;
        let token = self.cur_token().unwrap();
        match token.token_type {
            // parse l'expression jusqu'à ce que l'on rencontre Rparen
            TokenType::Lparen => self.parse_expression_list(TokenType::Rparen)
                .and_then(|args| Ok(box ast::Expression::FunCall(ident, args))),
            _ => error_unexpected_token(token),
        }
    }

    /// Parse une liste d'expression, c'est-à-dire une liste d'éléments séparés par des virgules.
    /// - Terminator: Le token qui termine la séquence.
    fn parse_expression_list(&mut self, terminator: token::TokenType)
        -> LResult<Vec<Box<ast::Expression>>> {
//        let vec = vec![];
//        match self.cur_token {
//            Some(ref t) if self.cur_token_is(t) => Ok(vec),
//        }
        unimplemented!()
    }

    /// Parse un literal (nombre, booléen, array, string).
    fn parse_literal(&mut self) -> LResult<ast::Literal> {
        match self.cur_token.as_ref().unwrap().token_type {
            TokenType::Boolean(_) => self.parse_boolean(),
            TokenType::Literal(_) => self.parse_string(),
            TokenType::Number(_) => self.parse_number().map(ast::Literal::from),
            TokenType::Lbracket => self.parse_array(),
            _ => error_unexpected_token(self.cur_token().unwrap())
        }
    }

    /// Parse un array.
    fn parse_array(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Lbracket =>
                self.parse_expression_list(TokenType::Rbracket).map(ast::Literal::from),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un booléen.
    fn parse_boolean(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Boolean(b) => Ok(ast::Literal::Boolean(b)),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un nombre.
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

    /// Parse une chaîne de caractères.
    fn parse_string(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Literal(st) => Ok(ast::Literal::String(st)),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un identifiant.
    fn parse_identifier(&mut self) -> LResult<ast::Identifier> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Identifier(st) => Ok(st),
            _ => error_unexpected_token(token)
        }
    }

    /// Parse un type.
    /// Aucun support pour les types génériques pour l'instant.
    fn parse_type(&mut self) -> LResult<ast::Type> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Identifier(name) => Ok(ast::Type { name }),
            _ => error_unexpected_token(token)
        }
    }
}

// TODO(berbiche): me remplir de test
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test() {

    }
}
