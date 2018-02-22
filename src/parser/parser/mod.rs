use ast;
use lexer::{Lexer, error::{Error, LResult}};
use token;

use std::collections::{HashMap, HashSet};

use phf;


/// Collection contenant les identifiants de ce `Parser`
//type Map<'a, 'b: 'a> = &'a StdHashMap<&'b str, token::Token>;
pub type Map<'a: 'b, 'b> = HashMap<&'b str, &'b str>;

/// Le `Parser` contient toutes les informations associés au "parsage" d'une entrée
#[derive(Debug)]
pub struct Parser<'a, 'b: 'a> {
    /// L'instance du Lexer a utilisé
    lexer: Lexer<'a>,
    /// Le `Token` sur lequel le `Parser` est présentement
    cur_token: Option<token::Token>,
    /// Le prochain `Token`
    peek_token: Option<token::Token>,
    /// Structure de données contenant les identifiants rencontrés
    identifiers: Map<'a, 'b>,
    /// Les erreurs rencontrés par le `Parser`
    errors: Vec<Error>,
}

impl<'a, 'b: 'a> Parser<'a, 'b> {
    /// Crée une nouvelle instance de `Parser` qui utilise le `Lexer` et le Map
    /// passés en arguments
    pub fn new(lexer: Lexer<'a>, identifiers: Map<'a, 'b>) -> Self {
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

    /// Permet d'ajouter un Map d'identifiant avant de `parse()`
    pub fn add_identifiers(&mut self, idents: Map<'a, 'b>) {
        self.identifiers.extend(idents);
    }

    /// Consomme le `Lexer` et le `Parser` et renvoie
    /// le AST du programme ou un vecteur de `Error`
    pub fn parse(mut self) -> Result<ast::Program, Vec<Error>> {
        // avance le parser pour que peek_token soit set
        self.advance_token();
        // avance le parser a nouveau pour que current_token soit set
        self.advance_token();

//        match
        unimplemented!()
    }

    /// Lit le prochain `Token` dans `peek_token` et avance le `cur_token`
    /// au `Token` de `peek_token`
    /// Nécessite d'être invoquer 2 fois initialement car
    fn advance_token(&mut self) {
        use token::TokenType;
        self.cur_token = &self.peek_token;

        // avance le peek_token au prochain token
        self.peek_token = match self.lexer.read_token() {
            Ok(token) => match token.token_type() {
                TokenType::EOF => None,
                TokenType::Illegal(st) => {
                    let error = Error::UnexpectedCharacter(*st, token.location().clone());
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

    /// Renvoie l'importance du token actuel
    fn get_precedence(&self) -> u8 {
        use self::token::TokenType::*;
        match self.cur_token {
            Some(ref token) => match *token.token_type() {
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
                Lparen => 200,
                Rparen => 200,
                Eq => 255,
                _ => 5,
            },
            _ => 0,
        }
    }
}
