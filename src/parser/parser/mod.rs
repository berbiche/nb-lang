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
use token::{self, Keyword, Number, Token, TokenType};

use std::convert::TryInto;
use std::collections::{HashMap, HashSet};
use std::mem;

/// Type représentant le niveau de précédence d'un lexème
type PrecedenceLevel = u8;

/// La plus basse précédence possible
const LOWEST_PRECEDENCE: PrecedenceLevel = PrecedenceLevel::min_value();
/// La plus haute précédence possible
const HIGHEST_PRECEDENCE: PrecedenceLevel = PrecedenceLevel::max_value();

/// Table contenant les opérateurs unaires
// TODO(berbiche): https://github.com/sfackler/rust-phf/issues/43
lazy_static! {
    static ref UNARY_OPERATOR_SET: HashSet<TokenType> = {
        let mut map = HashSet::new();
        map.insert(TokenType::Not);
        map
    };
}

/// Table contenant les opérateurs binaires/infixes et leur priorité
// TODO(berbiche): https://github.com/sfackler/rust-phf/issues/43
lazy_static! {
    static ref BINARY_OPERATOR_MAP: HashMap<TokenType, PrecedenceLevel> = {
        let mut map = HashMap::new();
        map.insert(TokenType::EqEq, 5);
        map.insert(TokenType::OrOr, 5);
        map.insert(TokenType::AndAnd, 5);
        map.insert(TokenType::Not, 10);
        map.insert(TokenType::Plus, 20);
        map.insert(TokenType::Minus, 20);
        map.insert(TokenType::Division, 25);
        map.insert(TokenType::Multiplication, 25);
        map.insert(TokenType::Modulo, 25);
        map.insert(TokenType::Power, 30);
        map.insert(TokenType::Lparen, LOWEST_PRECEDENCE);
        map.insert(TokenType::Rparen, LOWEST_PRECEDENCE);
        map
    };
}

/// Renvoie la priorité du token passé en argument dans une expression.
#[inline]
fn get_precedence(token: &Token) -> PrecedenceLevel {
    BINARY_OPERATOR_MAP
        .get(&token.token_type)
        .map(|x| *x)
        .unwrap_or(LOWEST_PRECEDENCE)
}

/// Renvoie si le `TokenType` est un opérateur binaire
#[inline]
fn is_binary_operator(token_type: &TokenType) -> bool {
    BINARY_OPERATOR_MAP.contains_key(token_type)
}

/// Renvoie si le `TokenType` est un opérateur unaire
#[inline]
fn is_unary_operator(token_type: &TokenType) -> bool {
    UNARY_OPERATOR_SET.contains(token_type)
}

/// Compare le variant de deux TokenType, renvoie le résultat
#[inline]
fn is_same_tokentype(lhs: &TokenType, rhs: &TokenType) -> bool {
    mem::discriminant(lhs) == mem::discriminant(rhs)
}

/// Renvoie une erreur de mot-clé réservé
#[inline]
fn error_reserved_keyword(token: Token) -> LResult<!> {
    Err(Error::ReservedKeyword(token.token_type, token.location))
}

/// Crée une erreur de `Token` inattendu `Error::UnexpectedToken`
///
/// La plupart des invocations de cette fonction devraient être changé pour l'appel
/// de la fonction LLVM intrinsèque `::std::intrinsics::unreachable()`,
/// cette dernière déclare un chemin qui ne devrait **JAMAIS** être atteint
/// (_undefined behaviour_).
/// Comme expliqué plus haut, dû au système de typage et la manière dont le parser
/// a été écrit, on connait le type de token lorsqu'une fonction précise se fait appeler,
/// mais il n'est toutefois pas possible de démontrer cela avec les `enum` de _Rust_
/// (au meilleur de ma connaissance (incluant les méthodes intrinsèques/unsafe))
#[inline]
fn error_unexpected_token(token: Token) -> LResult<!> {
    Err(Error::UnexpectedToken(token.token_type, token.location))
}

/// Crée une erreur `Error::UnexpectedEOF`
#[inline]
fn error_unexpected_eof<P>(pos: P) -> LResult<!>
where
    P: Into<token::PositionOrSpan>,
{
    Err(Error::UnexpectedEOF(pos.into()))
}

fn error_expected_token<S>(st: S, token: Token) -> LResult<!>
where
    S: Into<String>,
{
    Err(Error::ExpectedToken(
        st.into(),
        token.token_type,
        token.location,
    ))
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
        } else {
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

    /// Renvoie le `cur_token`, le remplaçant par la valeur de `peek_token`.
    #[inline]
    fn cur_token(&mut self) -> Option<Token> {
        let result = mem::replace(&mut self.cur_token, None);
        self.advance_token();
        result
    }

    /// Renvoie la priorité du `Token` actuel.
    #[inline]
    fn cur_precedence(&self) -> u8 {
        self.cur_token.as_ref().map_or(0, get_precedence)
    }

    /// Renvoie la priorité du prochain `Token`.
    #[inline]
    fn peek_precedence(&self) -> u8 {
        self.peek_token.as_ref().map_or(0, get_precedence)
    }

    /// Renvoie si le `cur_token` est le `TokenType` passé en argument.
    #[inline]
    fn cur_token_is(&self, kind: &TokenType) -> bool {
        match self.cur_token {
            Some(ref token) => is_same_tokentype(&token.token_type, kind),
            _ => false,
        }
    }

    /// Renvoie si le `peek_token` est le `TokenType` passé en argument.
    #[inline]
    fn peek_token_is(&self, kind: &TokenType) -> bool {
        match self.peek_token {
            Some(ref token) => is_same_tokentype(&token.token_type, kind),
            _ => false,
        }
    }

    /// Renvoie une erreur `Error::ExpectedToken` si le `cur_token`
    /// n'est pas celui désiré.
    fn expect_token(&self, kind: &TokenType) -> LResult<()> {
        match self.cur_token {
            Some(ref token) if is_same_tokentype(&token.token_type, kind) => Ok(()),
            _ => Err(Error::UnexpectedEOF(self.lexer.position().into())),
        }
    }

    /// Renvoie une erreur `Error::ExpectedToken` si le `cur_token`
    /// n'est pas un identifiant.
    #[inline]
    fn expect_ident(&self) -> LResult<()> {
        self.expect_token(&TokenType::Identifier(String::new()))
    }

    /// Parse un énoncé, quel qu'il soit et le renvoie.
    /// Si quoique se soit est illégal dans l'énoncé, une erreur est générée.
    fn parse_statement(&mut self) -> LResult<ast::Statement> {
        use token::{Keyword::{self, *}, TokenType::{self, *}};
        match self.cur_token.as_ref().unwrap().token_type {
            Keyword(Keyword::Let) | Keyword(Keyword::Const) => self.parse_variable_declaration(),
            Keyword(Keyword::Fun) => self.parse_function_declaration(),
            Keyword(Keyword::Return) => self.parse_return(),
            Keyword(Keyword::Unless) | Keyword(Keyword::If) => self.parse_conditional(),
            Keyword(Keyword::While) => self.parse_while_loop(),
            Keyword(Keyword::Reserved(_)) => error_reserved_keyword(self.cur_token().unwrap())?,
            _ => self.parse_expression_statement(),
        }
    }

    /*
        Section contenant le code pour "parser" les `Token`s
        en un "node" `ast`.
    */
    /// Parse une déclaration de variable.
    fn parse_variable_declaration(&mut self) -> LResult<ast::Statement> {
        use ast::Statement::VariableDeclaration;

        let declaration_keyword = {
            let token = self.cur_token().unwrap();
            match token.token_type {
                TokenType::Keyword(Keyword::Let) => Keyword::Let,
                TokenType::Keyword(Keyword::Const) => Keyword::Const,
                _ => error_unexpected_token(token)?,
            }
        };

        self.expect_ident()?;
        let variable = {
            let ident = self.parse_identifier()?;
            ast::Variable {
                name: ident,
                category: if self.cur_token_is(&TokenType::Colon) {
                    self.parse_type()?
                } else {
                    ast::Type {
                        name: String::new(),
                    }
                },
            }
        };

        if self.cur_token.is_none() {
            error_unexpected_eof(self.lexer.position())?
        }

        let value = self.parse_expression(LOWEST_PRECEDENCE)?;
        Ok(VariableDeclaration(declaration_keyword, variable, value))
    }

    /// Parse une déclaration de fonction.
    /// Aucun support pour les fonctions génériques.
    fn parse_function_declaration(&mut self) -> LResult<ast::Statement> {
        use self::TokenType::Identifier;
        self.advance_token(); // consomme le `fun`

        let identifier = match self.cur_token.as_ref() {
            Some(t) if is_same_tokentype(&t.token_type, &Identifier(String::new())) => {
                self.parse_identifier()?
            },
            _ => error_unexpected_eof(self.lexer.position())?,
        };

        self.expect_token(&TokenType::Lparen)?;
        self.advance_token();

        let parameters = self.parse_function_prototype()?;

        // parse le return type de la fonction
        let return_type = match self.cur_token.as_ref() {
            Some(Token {
                token_type: TokenType::Arrow,
                ..
            }) => {
                self.advance_token();
                self.expect_ident()?;
                let typ = self.parse_identifier()?;
                ast::Type {
                    name: typ.to_string(),
                }
            },
            // la flèche -> est optionnelle
            Some(_) => {
                self.advance_token();
                ast::Type {
                    name: String::new(),
                }
            },
            None => error_unexpected_eof(self.lexer.position())?,
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
                Some(Token {
                    token_type: TokenType::Identifier(_),
                    ..
                }) => self.parse_identifier()?,
                Some(_) => {
                    // pas un identifiant
                    let token = self.cur_token().unwrap();
                    error_expected_token("identifier", token)?
                },
                _ => error_unexpected_eof(self.lexer.position())?,
            };

            self.expect_token(&TokenType::Colon)?;
            self.advance_token();

            let typ = match self.cur_token() {
                Some(token) => match token.token_type {
                    TokenType::Identifier(st) => self.parse_type()?,
                    _ => error_expected_token("type", token)?,
                },
                _ => error_unexpected_eof(self.lexer.position())?,
            };

            prototype.push(ast::Variable {
                name: identifier,
                category: typ,
            });
        }
        self.advance_token(); // consomme le ')' fermant

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
        self.advance_token(); // consomme le '}' fermant

        Ok(block.into())
    }

    /// Parse un énoncé-expression.
    /// Une expression seule est un énoncé valide dans le langage.
    fn parse_expression_statement(&mut self) -> LResult<ast::Statement> {
        let expression = self.parse_expression(LOWEST_PRECEDENCE)?;
        self.expect_token(&TokenType::Semicolon)?;
        self.advance_token(); // consomme le ';'
        Ok(ast::Statement::Expression(expression))
    }

    /// Parse un retour de fonction.
    fn parse_return(&mut self) -> LResult<ast::Statement> {
        self.expect_token(&TokenType::Keyword(Keyword::Return))?;
        self.advance_token();

        let return_value = if self.cur_token_is(&TokenType::Semicolon) {
            self.advance_token();
            None
        } else {
            let expr = self.parse_expression(LOWEST_PRECEDENCE)?;
            if self.cur_token_is(&TokenType::Semicolon) {
                self.advance_token()
            }
            Some(expr)
        };

        Ok(ast::Statement::Return(return_value))
    }

    /// Parse un énoncé conditionnel.
    fn parse_conditional(&mut self) -> LResult<ast::Statement> {
        use self::TokenType::Keyword;
        use self::Keyword::{Else, Elseif, If, Unless};
        let keyword = match self.cur_token.as_ref().unwrap().token_type {
            Keyword(If) => If,
            Keyword(Else) => match self.peek_token_is(&Keyword(If)) {
                true => Elseif,
                false => Else,
            },
            Keyword(Unless) => Unless,
            _ => error_unexpected_token(self.cur_token().unwrap())?,
        };
        self.advance_token(); // consomme le If / Unless / Else restant

        let condition = match keyword {
            Else => None,
            _ => Some(self.parse_expression(LOWEST_PRECEDENCE)?),
        };

        self.expect_token(&TokenType::Lbrace)?;
        let block = self.parse_statement_block()?;

        Ok(ast::Statement::Conditional(keyword, condition, block))
    }

    /// Parse une boucle `while`.
    fn parse_while_loop(&mut self) -> LResult<ast::Statement> {
        self.advance_token(); // consomme le 'while'
        let condition = self.parse_expression(LOWEST_PRECEDENCE)?;
        self.expect_token(&TokenType::Lbrace)?;
        let block = self.parse_statement_block()?;

        Ok(ast::Statement::Loop(Keyword::While, Some(condition), block))
    }

    /// Parse une expression.
    ///
    /// La fonction va d'abord parser l'expression sous elle, pour se faire
    /// on pattern match les lexèmes pouvant se retrouver en début d'expression,
    /// si un résultat est trouvé, on invoque la fonction associé au pattern.
    ///
    /// Une idée serait de faire une table de correspondance entre les `TokenType` et
    /// des fonctions pour parser plutôt que de pattern match.
    /// Les avantages sont multiples: plus grande extensibilité, moins de code et
    /// plus simple à raisonner.
    /// La plus grande extensibilité permet d'ajouter des nouvelles fonctions de "parsage"
    /// au besoin sans avoir à modifier le code ici.
    ///
    /// Si on désire supporter le "parsage" de fonction suffixe/infixe, il faut l'ajouter ici.
    fn parse_expression(&mut self, precedence: PrecedenceLevel) -> LResult<Box<ast::Expression>> {
        use self::ast::Expression as ex;
        use self::TokenType as tt;

        // D'abord, on parse la première expression.
        // Puis, on regarde si le prochain lexème est un lexème permis
        // entre deux opérandes, si oui nous avons une expression binaire.
        let mut lhs = match self.cur_token.as_ref() {
            // Toutes les choses qui peuvent se retrouver en début d'expression
            Some(cur_token) => match cur_token.token_type {
                tt::Literal(_) | tt::Number(_) | tt::Boolean(_) | tt::Lbracket => {
                    box ex::Literal(self.parse_literal()?)
                },
                tt::Lparen => self.parse_paren_expression()?,
                tt::Identifier(_) => match self.peek_token_is(&tt::Lparen) {
                    true => self.parse_call_expression()?,
                    _ => box ex::Identifier(self.parse_identifier()?),
                },
                ref token if is_unary_operator(token) => self.parse_prefix_expression()?,
                _ => {
                    let token = self.cur_token().unwrap();
                    error_unexpected_token(token)?
                },
            },
            None => error_unexpected_eof(self.lexer.position())?,
        };

        // Tant que nous n'avons pas atteint la fin de l'expression
        // nous collectons l'expression
        while !self.cur_token_is(&tt::Semicolon) && precedence < self.peek_precedence() {
            match self.cur_token.as_ref() {
                // Support pour les expressions suffixes peuvent être ajoutés ici
                // Some(cur_token) if is_unary_operator(&cur_token.token_type)
                Some(cur_token) if is_binary_operator(&cur_token.token_type) => {
                    lhs = self.parse_binary_expression(lhs)?
                },
                Some(_) => {
                    let token = self.cur_token().unwrap();
                    error_expected_token("opérateur binaire", token)?
                },
                None => error_unexpected_eof(self.lexer.position())?,
            }
        }

        // l'expression résultante est la racine d'une ou plusieurs expressions
        Ok(lhs)
    }

    /// Parse une expression entre parenthèses.
    fn parse_paren_expression(&mut self) -> LResult<Box<ast::Expression>> {
        self.advance_token(); // consomme le '('
                              // par défaut, la priorité la plus faible car nous sommes en début d'expression
        let expr = self.parse_expression(LOWEST_PRECEDENCE)?;
        self.expect_token(&TokenType::Rparen);
        self.advance_token(); // consomme le ')'

        Ok(expr)
    }

    /// Parse une expression prefix.
    fn parse_prefix_expression(&mut self) -> LResult<Box<ast::Expression>> {
        // consomme l'opérateur
        let operator = self.cur_token().unwrap().token_type;
        // converti en UnaryOperator
        let operator = operator.try_into().unwrap();

        let expr = self.parse_expression(LOWEST_PRECEDENCE)?;

        Ok(box ast::Expression::UnaryExpression(expr, operator))
    }

    /// Parse une expression binaire.
    /// Plus d'information sur ce qu'est une expression binaire dans `ast::BinaryOperator`.
    fn parse_binary_expression(
        &mut self,
        lhs: Box<ast::Expression>,
    ) -> LResult<Box<ast::Expression>> {
        // priorité de l'opérateur actuel
        let precedence = self.cur_precedence();
        // consomme l'opérateur
        let operator = self.cur_token().unwrap().token_type;
        // converti en BinaryOperator
        let operator = operator.try_into().unwrap();

        let rhs = self.parse_expression(precedence)?;

        Ok(box ast::Expression::BinaryExpression(lhs, operator, rhs))
    }

    /// Parse un appel de fonction.
    fn parse_call_expression(&mut self) -> LResult<Box<ast::Expression>> {
        let ident = self.parse_identifier()?;
        let token = self.cur_token().unwrap();
        match token.token_type {
            // parse l'expression jusqu'à ce que l'on rencontre Rparen
            TokenType::Lparen => self.parse_expression_list(TokenType::Rparen)
                .and_then(|args| Ok(box ast::Expression::FunCall(ident, args))),
            _ => error_unexpected_token(token)?,
        }
    }

    /// Parse une liste d'expression, c'est-à-dire une liste d'éléments séparés par des
    /// virgules.
    fn parse_expression_list(
        &mut self,
        terminator: token::TokenType,
    ) -> LResult<Vec<Box<ast::Expression>>> {
        let mut expressions = vec![];
        if self.cur_token_is(&terminator) {
            self.advance_token();
            return Ok(expressions);
        }

        while !self.cur_token_is(&terminator) {
            let expr = self.parse_expression(LOWEST_PRECEDENCE)?;

            self.expect_token(&TokenType::Comma)?;
            self.advance_token();

            expressions.push(expr)
        }
        self.advance_token(); // consomme le terminator

        Ok(expressions)
    }

    /// Parse un literal (nombre, booléen, array, string).
    fn parse_literal(&mut self) -> LResult<ast::Literal> {
        match self.cur_token.as_ref().unwrap().token_type {
            TokenType::Boolean(_) => self.parse_boolean(),
            TokenType::Literal(_) => self.parse_string(),
            TokenType::Number(_) => self.parse_number().map(ast::Literal::from),
            TokenType::Lbracket => self.parse_array(),
            _ => error_unexpected_token(self.cur_token().unwrap())?,
        }
    }

    /// Parse un array.
    fn parse_array(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Lbracket => self.parse_expression_list(TokenType::Rbracket)
                .map(ast::Literal::from),
            _ => error_unexpected_token(token)?,
        }
    }

    /// Parse un booléen.
    fn parse_boolean(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Boolean(b) => Ok(ast::Literal::Boolean(b)),
            _ => error_unexpected_token(token)?,
        }
    }

    /// Parse un nombre.
    fn parse_number(&mut self) -> LResult<ast::Number> {
        // parse un numéro dans la base donnée
        // (String -> i32|i64 -> ast::Number)
        // Présentement, il n'est pas possible de déterminer la cause d'erreur
        // ... car les enum std::num::IntErrorKind et std::num::FloatErrorKind sont privés
        // ... `parse`/`from_str_radix` renvoie
        // ... `ParseIntError { kind: IntErrorKind/FloatErrorKind }`
        fn parse_with_base(
            num: String,
            base: u32,
            location: token::PositionOrSpan,
        ) -> LResult<ast::Number> {
            let num = num.replace("_", "");

            // e
            let mut number = i32::from_str_radix(num.as_ref(), base).map(ast::Number::from);
            if number.is_err() {
                number = i64::from_str_radix(num.as_ref(), base).map(ast::Number::from);
            }
            number.map_err(|_err| Error::InvalidNumber(num, location))
        }

        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Number(number) => match number {
                Number::Binary(num) => parse_with_base(num, 2, token.location),
                Number::Octal(num) => parse_with_base(num, 8, token.location),
                Number::Hexadecimal(num) => parse_with_base(num, 16, token.location),
                Number::Decimal(num) => {
                    let num = num.replace("_", "");
                    // Converti nombre -> ast::Number
                    let success = num.parse::<i32>()
                        .map(ast::Number::from)
                        .or_else(|_| num.parse::<i64>().map(ast::Number::from))
                        .or_else(|_| num.parse::<f64>().map(ast::Number::from));
                    // ne compile pas, problème avec borrowck
                    //                    .map_err(|_| Error::InvalidNumber(num, token.location));
                    // alternative
                    match success {
                        Err(_) => Err(Error::InvalidNumber(num, token.location)),
                        Ok(t) => Ok(t),
                    }
                },
            },
            _ => error_unexpected_token(token)?,
        }
    }

    /// Parse une chaîne de caractères.
    fn parse_string(&mut self) -> LResult<ast::Literal> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Literal(st) => Ok(ast::Literal::String(st)),
            _ => error_unexpected_token(token)?,
        }
    }

    /// Parse un identifiant.
    fn parse_identifier(&mut self) -> LResult<ast::Identifier> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Identifier(st) => Ok(st),
            _ => error_unexpected_token(token)?,
        }
    }

    /// Parse un type.
    /// Aucun support pour les types génériques pour l'instant.
    fn parse_type(&mut self) -> LResult<ast::Type> {
        let token = self.cur_token().unwrap();
        match token.token_type {
            TokenType::Identifier(name) => Ok(ast::Type { name }),
            _ => error_unexpected_token(token)?,
        }
    }
}

// TODO(berbiche): me remplir de test
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test() {}
}
