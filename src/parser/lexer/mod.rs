use self::error::{Error, LResult};
use token::*;

use itertools::Itertools;

use std::iter::Peekable;
use std::result;
use std::str::{Chars, FromStr};
use std::vec::Vec;

pub mod error;

/// Le Lexer est un wrapper sur un itérateur qui lit les caractères pour former des
/// lexèmes.
/// Celui-ci fournit plusieurs méthodes aisant l'utilisation de l'itérateur.
/// Le Lexer est donc lui-même un itérateur, permettant le streaming des `token::Token`
#[derive(Debug)]
pub struct Lexer<'a> {
    /// Caractère courant dans la séquence de caractères
    current_char: Option<char>,
    /// Lexème courant dans le vecteur de token
    current_token: Option<Token>,
    /// L'entrée à parse, une séquence de caractères itérable
    input: Peekable<Chars<'a>>,
    /// Position actuelle dans le programme
    /// `line` est incrémenté chaque fois qu'un caractère de newline est rencontré
    /// en prenant en considération le fait que certains systèmes d'exploitation
    /// utilise plusieurs caractères pour représenter une nouvelle ligne
    position: Position,
}

impl<'a> Lexer<'a> {
    /// Construit un Lexer depuis une chaîne de caractères
    pub fn new<S>(input: S) -> Self
        where
            S: Into<&'a str>,
    {
        let mut lexer = Lexer {
            current_char: None,
            current_token: None,
            input: input.into().chars().peekable(),
            position: Position { column: 0, line: 1 },
        };
        lexer.read(); // avance au premier caractère
        lexer
    }

    /// Construit le prochain `token::Token` et le renvoie
    /// Renvoie `None` si la fin de la séquence est atteint
    /// Validation minimale se fait ici, c'est-à-dire que les nombres ne sont pas validés
    // TODO: Convertir la plus part de cette tâche en celle d'un macro
    pub fn read_token(&mut self) -> LResult<Token> {
        use token::{TokenType::*, Keyword::{self, *}, Number::*};

        // saute les espaces blancs
        // TODO: M'enlever une fois que le bug avec skip_whitespace sera résolu
        loop {
            match self.current_char {
                Some(ch) if ch.is_whitespace() => self.read(),
                _ => break,
            };
        }

        let result = match self.current_char {
            None => token!(EOF, self.position),
            Some(ch) => match ch {
                '+' => token!(Plus, self.position),
                '%' => token!(Modulo, self.position),
                '^' => token!(Power, self.position),
                '*' => token!(Multiplication, self.position),
                '-' => match self.peek() {
                    Some(&ch) if ch == '>' => {
                        let begin = self.position;
                        self.read();
                        token!(Arrow, begin => self.position)
                    },
                    _ => token!(Minus, self.position),
                },
                '/' => match self.peek() {
                    Some(&ch) if ch == '*' => { // commentaire
                        let begin = self.position;
                        let st = self.read_comment();
                        token!(Comment(st), begin => self.position)
                    },
                    _ => token!(Division, self.position),
                },
                '=' => match self.peek() {
                    Some(&ch) if ch == '=' => { // EqEq
                        let begin = self.position;
                        self.read();
                        token!(EqEq, begin => self.position)
                    },
                    _ => token!(Eq, self.position),
                },
                '!' => match self.peek() {
                    Some(&ch) if ch == '=' => { // non égal
                        let begin = self.position;
                        self.read();
                        token!(NotEq, begin => self.position)
                    },
                    _ => token!(Not, self.position),
                },
                '<' => match self.peek() {
                    Some(&ch) if ch == '=' => { // plus petit que ou égal
                        let begin = self.position;
                        self.read();
                        token!(LtEq, begin => self.position)
                    },
                    _ => token!(Lt, self.position)
                },
                '>' => match self.peek() {
                    Some(&ch) if ch == '=' => { // plus grand que ou égal
                        let begin = self.position;
                        self.read();
                        token!(GtEq, begin => self.position)
                    },
                    _ => token!(Gt, self.position)
                },
                '|' => match self.peek() {
                    Some(&ch) if ch == '|' => { // ou ou
                        let begin = self.position;
                        self.read();
                        token!(OrOr, begin => self.position)
                    },
                    _ => token!(Or, self.position),
                },
                '&' => match self.peek() {
                    Some(&ch) if ch == '&' => { // et et
                        let begin = self.position;
                        self.read();
                        token!(AndAnd, begin => self.position)
                    },
                    _ => token!(And, self.position),
                },
                ',' => token!(Comma, self.position),
                ':' => token!(Colon, self.position),
                ';' => token!(Semicolon, self.position),
                '(' => token!(Lparen, self.position),
                ')' => token!(Rparen, self.position),
                '{' => token!(Lbrace, self.position),
                '}' => token!(Rbrace, self.position),
                '[' => token!(Lbracket, self.position),
                ']' => token!(Rbracket, self.position),
                '_' => token!(Underscore, self.position),
                '"' => {
                    let begin = self.position;
                    let st = self.read_string()?;
                    token!(Literal(st), begin => self.position)
                },
                ch if ch.is_alphabetic() => { // identifiant ou keyword
                    let begin = self.position;
                    let ident = self.read_identifier();

                    if ident == "true" {
                        token!(Boolean(true), begin => self.position)
                    }
                    else if ident == "false" {
                        token!(Boolean(false), begin => self.position)
                    }
                    else {
                        match Keyword::lookup(ident.as_ref()) {
                            Some(token) => token!(token, begin => self.position),
                            None => token!(Identifier(ident), begin => self.position),
                        }
                    }
                },
                ch if ch.is_decimal_digit() => { // lit un nombre décimal/octal/etc.
                    let begin = self.position;
                    match (ch, self.peek()) {
                        ('0', Some(&peeked)) => match &peeked {
                            'b' => { // binaire
                                self.read();
                                let st = self.read_number();
                                token!(Binary(st), begin => self.position)
                            },
                            'o' => { // octal
                                self.read();
                                let st = self.read_number();
                                token!(Octal(st), begin => self.position)
                            },
                            'x' => { // hexadécimal
                                self.read();
                                let st = self.read_number();
                                token!(Hexadecimal(st), begin => self.position)
                            },
                            _ => token!(Decimal(self.read_number()), begin => self.position),
                        },
                        _ => token!(Decimal(self.read_number()), begin => self.position),
                    }
                },
                _ => token!(Illegal(ch.to_string()), self.position),
            }
        };

        // avance au prochain caractère
        self.read();
        result
    }

    /// Getter pour la position du lexer dans la séquence
    #[inline]
    pub fn position(&self) -> Position {
        self.position
    }

    /// Renvoie si le caractère actuel est celui passé en argument
    #[inline]
    fn current_char_is(&self, other: char) -> bool {
        self.current_char == Some(other)
    }

    /// Permet de voir le prochain caractère sans consommer le caractère
    /// renvoie `None` si la fin de la séquence est atteinte
    #[inline]
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// Renvoie le prochain caractère, le consommant de l'itérateur
    /// renvoie `None` si la fin de la séquence est atteinte
    /// Met à jour la position du lexer dans la séquence
    // TODO(berbiche): Incrémenter la ligne au prochain call de `read`
    // TODO(berbiche): ...plutôt que lorsqu'une fin de ligne est rencontré
    fn read(&mut self) -> Option<char> {
        let previous = self.current_char;
        let current = self.input.next();

        if let Some(current) = current {
            if let Some(previous) = previous {
                // si nous n'avons pas une séquence CRLF
                if is_newline(&previous) &&
                    !(previous == '\u{000D}' && current == '\u{000A}') {
                    self.position.line += 1;
                    self.position.column = 0;
                }
            }
            self.position.column += 1;
        }

        self.current_char = current;
        self.current_char
    }

    /// Permet de lire un identifiant contenant optionnellement un '?'
    /// (question mark) à la fin
    fn read_identifier(&mut self) -> String {
        let mut st: String = self.current_char.unwrap().to_string();
        {
            let iter = self.input
                .peeking_take_while(|ch| ch.is_alphabetic() || *ch == '_');
            st.extend(iter);
        }

        // permet d'avoir un point d'interrogation à la fin d'un identifiant
        if let Some(&ch) = self.peek() {
            if ch == '?' {
                self.read();
                st.push(ch);
            }
        }
        st
    }

    /// Lit un commentaire
    fn read_comment(&mut self) -> String {
        if self.current_char_is('/') { // lit un commentaire de ligne
            self.input.peeking_take_while(is_newline).collect()
        }
        else { // lit un commentaire de bloc
            let mut last_ch: char = 0 as u8 as char;
            self.input
                .peeking_take_while(|ch| {
                    if last_ch != '*' && *ch != '/' {
                        last_ch = ch.clone();
                        true
                    }
                    else {
                        false
                    }
                })
                .collect()
        }
    }

    /// Permet de lire un nombre
    #[inline]
    fn read_number(&mut self) -> String {
        let mut st = self.current_char.unwrap().to_string();
        let iter = self.input.peeking_take_while(|ch| ch.is_hexadecimal_digit() || *ch == '_');
        st.extend(iter);
        st
    }

    /// Lit une chaîne de caractères jusqu'à un '"' non-échappé
    fn read_string(&mut self) -> LResult<String> {
        // nous voulons itérer sur la séquence jusqu'à ce que nous trouvions
        // le caractère '"' qui n'a pas le caractère d'échappe '\\' avant
        // et que ce caractère d'échappe n'est pas échappé

        // prend le premier caractère qui est '"'
        let mut st = self.current_char.unwrap().to_string();

        // nous devons connaître le caractère précédent pour savoir si échappé
        let mut previous_ch = '\0';
        // pour avoir la bonne position avec les newline
        while let Some(current_ch) = self.read() {
            if is_newline(&current_ch) {
                return Err(Error::UnterminatedString(self.position.into()))
            }

            // les caractères de contrôle sont interdits
            if current_ch.is_control() {
                return Err(Error::InvalidString(st, self.position.into()))
            }

            st.push(current_ch);

            // si l'échappe est échappé
            if previous_ch == '\\' && current_ch == '\\' {
                previous_ch = '\0';
                continue;
            }

            if previous_ch != '\\' && current_ch == '"' {
                return Ok(st);
            }

            previous_ch = current_ch;
        }

        Err(Error::UnexpectedEOF(self.position.into()))
    }

    /// Saute les espaces-blancs, incluant le retour à la ligne
    #[inline]
    fn skip_whitespace(&mut self) {
        self.input
            .by_ref()
            .skip_while(|ch| ch.is_whitespace());
    }
}

/*
    Fonctions privées, simplement des raccourçis
*/
trait IsDigit {
    /// Renvoie si le caractère actuel est un chiffre décimal
    fn is_decimal_digit(&self) -> bool;
    /// Renvoie si le caractère actuel est un chiffre octal
    fn is_octal_digit(&self) -> bool;
    /// Renvoie si le caractère actuel est un chiffre hexadécimal
    fn is_hexadecimal_digit(&self) -> bool;
}

impl IsDigit for char {
    #[inline]
    fn is_decimal_digit(&self) -> bool {
        self.is_digit(10)
    }

    #[inline]
    fn is_octal_digit(&self) -> bool {
        self.is_digit(8)
    }

    #[inline]
    fn is_hexadecimal_digit(&self) -> bool {
        self.is_digit(16)
    }
}


/// Renvoie `true` si le caractère est une fin de ligne
/// Supporte les fins de ligne de plusieurs OS
#[inline]
fn is_newline(ch: &char) -> bool {
    match *ch {
        '\u{000A}'...'\u{000D}' | '\u{0085}' | '\u{2028}' | '\u{2029}' | '\0' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(Nicolas): Me documenter
    macro_rules! test_lexer {
        (@decl $lexer:ident, $input:expr) => (let mut $lexer = Lexer::new($input););

        (@assert $fn_name:ident, $lexer:ident, $result:expr) => (
            assert_eq!($result, $lexer.$fn_name());
        );

        (@do $lexer:ident, $skip_amount:expr) => (
            for _ in 0..$skip_amount {
                $lexer.read();
            }
        );

        // test_lexer!(fn_name, ["" => "", "" => ""])
        ($fn_name:ident, [
            $( $input:expr => $result:expr, )+
        ]) => (
            $(
                test_lexer!(@decl lexer, $input);
                test_lexer!(@assert $fn_name, lexer, $result);
            )*
        );

        // test_lexer!(fn_name, 6, ["" => "", "" => ""]
        ($fn_name:ident, $skip_amount:expr, [
            $( $input:expr => $result:expr, )+
        ]) => (
            $(
                test_lexer!(@decl lexer, $input);
                test_lexer!(@do lexer, $skip_amount);
                test_lexer!(@assert $fn_name, lexer, $result);
            )*
        );

        ([ $( $input:expr => [ $( $e:expr ),+ ], )+ ]) => (
            $(
                test_lexer!(@decl lexer, $input);
                let expected: &[TokenType] = &[ $($e.into()),* ];
                let mut tokens = Vec::new();
                loop {
                    match lexer.read_token() {
                        Ok(token) => match token.token_type() {
                            EOF => break,
                            _ => tokens.push(token),
                        },
                        Err(error) => panic!("Erreur: {:?}", error),
                    };
                }

                let token_types: Vec<_> = tokens.iter()
                    .map(|token| token.token_type())
                    .cloned()
                    .collect();

                assert_eq!(expected, &token_types[..]);
            )*
        );
    }

    #[test]
    fn position() {
        test_lexer!(position, 7, [
            "1234567" => Position::new(1, 7),
        ]);
    }

    #[test]
    fn position_newline() {
        test_lexer!(position, 4, [
            "\r\n\r\n" => Position::new(2, 2),
            "\n\n\r\n" => Position::new(3, 2),
            "\n\r\n\r" => Position::new(3, 1),
            "\r\r\r\n" => Position::new(3, 2),
            "\n\n\n\n" => Position::new(4, 1),
            "\r\r\r\r" => Position::new(4, 1),
        ]);
    }

    #[test]
    fn read_string() {
        test_lexer!(read_string, 6, [
            r#"voici "une longue chaîne de caractères valide"<-FIN"#
                => Ok(r#""une longue chaîne de caractères valide""#.to_string()),
        ]);
    }

    #[test]
    fn read_string_escaped() {
        test_lexer!(read_string, [
            r#""longue chaîne doublement \" échappé \\"<-FIN"#
                => Ok(r#""longue chaîne doublement \" échappé \\""#.to_string()),
        ]);
    }

    #[test]
    fn read_string_with_newline_should_error() {
        test_lexer!(read_string, [
            "\"ouah j'ai un retour à la ligne juste ici ->\n\"<-FIN"
                => Err(Error::UnterminatedString(Position::new(1, 45).into())),
        ]);
    }

    #[test]
    fn read_identifier() {
        test_lexer!(read_identifier, [
            "allo-ne me lit pas" => "allo",
        ]);
    }

    #[test]
    fn read_identifier_with_question_mark() {
        test_lexer!(read_identifier, [
            "test? allo" => "test?",
        ]);
    }

    #[test]
    fn tokenize() {
        use token::{
            Token,
            TokenType::{self, *},
            Keyword::{self, *},
            ReservedKeyword::{self, *},
            Number::{self, *},
        };

        test_lexer!([
            "let input = 5;" => [
                Let,
                Identifier("input".to_string()),
                Eq,
                Decimal("5".to_string()),
                Semicolon
            ],
            "fun fonction() { allo }" => [
                Fun,
                Identifier("fonction".to_string()),
                Lparen,
                Rparen,
                Lbrace,
                Identifier("allo".to_string()),
                Rbrace
            ],
//            "struct Nicolas {
//                x: int,
//                y: int,
//            }" => [
//                Struct,
//                Identifier("Nicolas".to_string()),
//                Lbrace,
//                Identifier("x".to_string()),
//                Colon,
//                Identifier("int".to_string()),
//                Comma,
//                Identifier("y".to_string()),
//                Colon,
//                Identifier("int".to_string()),
//                Comma,
//                Rbrace
//            ],
        ]);
    }
}

