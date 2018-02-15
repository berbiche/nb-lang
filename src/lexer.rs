use token::{Token, TokenType};

use std::cmp::Ordering;
use std::fmt;
use std::iter::Peekable;
use std::result;
use std::str::Chars;


/// Un type spécialisé pour les erreurs du lexer
pub type Result<T> = result::Result<T, Error>;

// FIXME(Nicolas): Me remplir d'encore plus d'erreurs
#[derive(Debug, Eq, Fail, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Identifiant invalide
    #[fail(display = "Identifiant invalide: '{}' à {}", ident, position)]
    InvalidIdentifier { ident: String, position: Position },
    /// Une chaîne de caractère invalide dans l'entrée
    #[fail(display = "Chaîne de caractères invalide: '{}' à {}", st, position)]
    InvalidString {  st: String, position: Position },
    /// Début de chaîne de caractères manquant '"'
    #[fail(display = "Début de chaîne de caractères manquant à {}", position)]
    MissingStringBeginning { position: Position },
    /// End-of-file atteint avant la fin de l'opération désiré
    #[fail(display = "End-of-File atteint avant la fin de la séquence désiré à {}", position)]
    UnexpectedEOF { position: Position },
    /// Chaîne de caractères non-terminée, peut-être dû à un EOF comme autre chose
    #[fail(display = "Chaîne de caractères n'est pas terminée à {}", position)]
    UnterminatedString { position: Position },
}

/// Représente une position dans un programme
/// Peut être employé pour attacher de l'information sur un lexème ou autre
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    /// Ligne
    line: usize,
    /// Colonne
    column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position {
            line,
            column,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Le Lexer est un wrapper sur un itérateur qui lit les caractères pour former des
/// lexèmes.
/// Celui-ci fournit plusieurs méthodes aisant l'utilisation de l'itérateur
#[derive(Debug)]
pub struct Lexer<'a> {
    /// Caractère courant dans la séquence de caractères
    current_char: Option<char>,
    /// Lexème courant dans le vecteur de token
    current_token: Option<Token<'a>>,
    /// L'entrée à parse, une séquence de caractères itérable
    input: Peekable<Chars<'a>>,
    /// Position actuelle dans le programme
    /// `line` est incrémenté chaque fois qu'un caractère de newline est rencontré
    /// en prenant en considération le fait que certains systèmes d'exploitation
    /// utilise plusieurs caractères pour représenter une nouvelle ligne
    position: Position,
    /// Vecteur contenant les lexèmes
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    /// Construit un Lexer depuis une chaîne de caractères
    pub fn new<S>(input: S) -> Self
    where
        S: Into<&'a str>,
    {
        let input = input.into();
        Lexer {
            current_char: None,
            current_token: None,
            input: input.chars().peekable(),
            position: Position { column: 0, line: 1 },
            tokens: Vec::new(),
        }
    }

    pub fn current_token() -> Option<&'a Token<'a>> {
        unimplemented!()
    }

    pub fn peek_token() -> Option<&'a Token<'a>> {
        unimplemented!()
    }

    pub fn read_token() -> Option<Token<'a>> {
        unimplemented!()
    }

    /// Renvoie le caractère courant dans la séquence de caractères
    /// renvoie `None` si la fin de la séquence est atteinte
    fn current_char(&self) -> Option<char> {
        self.current_char
    }

    /// Renvoie la position actuelle dans le programme du lexer
    fn position(&self) -> &Position {
        &self.position
    }

    /// Permet de voir le prochain caractère sans consommer le caractère
    /// renvoie `None` si la fin de la séquence est atteinte
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// Renvoie le prochain caractère, le consommant de l'itérateur
    /// renvoie `None` si la fin de la séquence est atteinte
    fn read(&mut self) -> Option<char> {
        let current = self.input.next();

        if let Some(current) = current {
            if is_newline(&current) {
                if let Some(previous) = self.current_char {
                    // si nous n'avons pas une séquence CRLF
                    if !(previous == '\u{000D}' && current == '\u{000A}') {
                        self.position.line += 1;
                        self.position.column = 0;
                    }
                }
            }
            else {
                self.position.column += 1;
            }
        }

        self.current_char = current;
        self.current_char
    }

    /// Saute les espaces-blancs, incluant le retour à la ligne
    fn skip_whitespace(&mut self) {
        self.skip_while(|ch| ch.is_whitespace());
    }

    /// Lit un identifiant et le renvoie
    ///
    /// ```ignore
    /// use nb_parser::lexer::Lexer;
    ///
    /// let input = "identifiant_valide fin";
    /// let mut lexer = Lexer::new(input);
    ///
    /// let expected = Ok("identifiant_valide".to_string());
    /// assert_eq!(expected, lexer.read_identifier());
    /// ```
    fn read_identifier(&mut self) -> Option<String> {
        let pos = self.position;
        let mut st = match self.take_while(|ch| ch.is_alphabetic() || *ch == '_') {
            Some(st) => st,
            None => return None,
        };

        // permet d'avoir un point d'interrogation à la fin d'un identifiant
        let contains_question_mark = match self.peek() {
            Some(ch) if *ch == '?' => true,
            _ => false,
        };

        if contains_question_mark {
            st.push(self.read().unwrap());
        }

        Some(st)
    }

    /// Permet de lire des nombres décimaux
    ///
    /// ```ignore
    /// use nb_parser::lexer::Lexer;
    ///
    /// let input = "1920193210";
    /// let mut lexer = Lexer::new(input);
    ///
    /// let expected = Some("1920193210".to_string());
    /// assert_eq!(expected, lexer.read_decimal());
    /// ```
    fn read_decimal(&mut self) -> Option<String> {
        self.take_while(is_digit)
    }

    /// Permet de lire des nombres hexadécimaux
    ///
    /// ```ignore
    /// use nb_parser::lexer::Lexer;
    ///
    /// let input = "F0A58B";
    /// let mut lexer = Lexer::new(input);
    ///
    /// let expected = Some("F0A58B".to_string());
    /// assert_eq!(expected, lexer.read_hexadecimal());
    fn read_hexadecimal(&mut self) -> Option<String> {
        self.take_while(is_hex)
    }

    /// Permet de lire des nombres octaux
    ///
    /// ```ignore
    /// use nb_parser::lexer::Lexer;
    ///
    /// let input = "777123045689";
    /// let mut lexer = Lexer::new(input);
    ///
    /// let expected = Some("7771230456".to_string());
    /// assert_eq!(expected, lexer.read_octal());
    fn read_octal(&mut self) -> Option<String> {
        self.take_while(is_octal)
    }

    /// Lit une chaîne de caractères jusqu'à un '"' non-échappé
    ///
    /// ```ignore
    /// use nb_parser::lexer::Lexer;
    ///
    /// let input = r#""chaîne de caractères""#;
    /// let mut lexer = Lexer::new(input);
    ///
    /// let expected = Ok(r#""chaîne de caractères""#.to_string());
    /// assert_eq!(expected, lexer.read_string());
    /// ```
    fn read_string(&mut self) -> Result<String> {
        // nous voulons itérer sur la séquence jusqu'à ce que nous trouvions
        // le caractère '"' qui n'a pas le caractère d'échappe '\\' avant
        // et que ce caractère d'échappe n'est pas échappé
        let mut st = String::new();
        // le premier '"'
        if let Some(ch) = self.read() {
            st.push(ch);
        }

        // nous devons connaître le caractère précédent pour savoir si échappé
        let mut previous_ch = '\0';
        while let Some(current_ch) = self.read() {
            let pos = self.position;

            if current_ch == '\0' {
                return Err(Error::UnexpectedEOF { position: pos })
            }
            // les caractères de contrôle sont interdits
            if !is_newline(&current_ch) && current_ch.is_control() {
                return Err(Error::InvalidString { st, position: pos });
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

        Err(Error::UnterminatedString { position: self.position })
    }

    /// Notre version de skip_while qui ne consomme pas le caractère
    /// si celui-ci ne respect pas le prédicat.
    /// Consomme tous les caractères correspondant au prédicat jusqu'à
    /// avoir atteint un caractère qui ne répond pas au prédicat
    fn skip_while<Predicate>(&mut self, predicate: Predicate)
    where
        Predicate: Fn(&char) -> bool,
    {
        loop {
            match self.peek() {
                Some(ch) if predicate(ch) => (),
                _ => return,
            };
            self.read();
        }
    }

    /// Notre version de take_while qui ne consomme pas le caractère
    /// s'il ne correspond pas au prédicat passé en paramètre.
    /// Consomme tous les caractères correspondant au prédicat jusqu'à
    /// avoir atteint un caractère qui ne répond pas au prédicat
    ///
    /// Renvoie une chaîne de caractère ou None si aucun caractère ne correspond
    /// au prédicat
    fn take_while<Predicate>(&mut self, predicate: Predicate) -> Option<String>
    where
        Predicate: Fn(&char) -> bool,
    {
        let mut accumulator = String::new();
        loop {
            match self.peek() {
                Some(ch) if predicate(ch) => (),
                _ => match accumulator.is_empty() {
                    true => return None,
                    false => return Some(accumulator),
                },
            };
            accumulator.push(self.read().unwrap());
        }
    }
}

/// Renvoie si le caractère actuel est un chiffre décimal
#[inline]
fn is_digit(ch: &char) -> bool {
    ch.is_digit(10)
}

/// Renvoie si le caractère actuel est un chiffre hexadécimal
#[inline]
fn is_hex(ch: &char) -> bool {
    ch.is_digit(16)
}

/// Renvoie si le caractère actuel est un chiffre octal
#[inline]
fn is_octal(ch: &char) -> bool {
    ch.is_digit(8)
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
    // TODO(Nicolas): Me réécrire sous forme d'un "TT-Muncher"
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
        (@inner $fn_name:ident,
            $( $input:expr, $result:expr, )+
        ) => (
            $(
                test_lexer!(@decl lexer, $input);
                test_lexer!(@assert $fn_name, lexer, $result);
            )*
        );

        // test_lexer!(fn_name, 6, ["" => "", "" => ""]
        (@inner-num $fn_name:ident, $skip_amount:expr,
            $( $input:expr, $result:expr, )+
        ) => (
            $(
                test_lexer!(@decl lexer, $input);
                test_lexer!(@do lexer, $skip_amount);
                test_lexer!(@assert $fn_name, lexer, $result);
            )*
        );

        // test_lexer!(fn_name, ["" => ok "", "" => ok ""])
        ($fn_name:ident, [
            $( $input:expr => ok $result:expr, )+
        ]) => (
            test_lexer!(@inner $fn_name, $($input, Ok($result.to_string()),)*);
        );

        // test_lexer!(fn_name, ["" => some "", "" => some ""])
        ($fn_name:ident, [
            $( $input:expr => some $result:expr, )+
        ]) => (
            test_lexer!(@inner $fn_name, $($input, Some($result.to_string()),)*);
        );

        // test_lexer!(fn_name, ["" => "", "" => ""])
        ($fn_name:ident, [
            $( $input:expr => $result:expr, )+
        ]) => (
            test_lexer!(@inner $fn_name, $($input, $result,)*);
        );

        // test_lexer!(fn_name, 1, ["" => "", ok "" => ok "", ])
        ($fn_name:ident, $skip_amount:expr, [
            $( $input:expr => ok $result:expr, )+
        ]) => (
            test_lexer!(@inner-num $fn_name, $skip_amount, $($input, Ok($result.to_string()),)*);
        );

        // test_lexer!(fn_name, 1, ["" => some "", "" => some ""])
        ($fn_name:ident, $skip_amount:expr, [
            $( $input:expr => some $result:expr, )+
        ]) => (
            test_lexer!(@inner-num $fn_name, $skip_amount, $($input, Some($result.to_string()),)*);
        );

        // test_lexer!(fn_name, 1, ["" => "", "" => ""])
        ($fn_name:ident, $skip_amount:expr, [
            $( $input:expr => $result:expr, )+
        ]) => (
            test_lexer!(@inner-num $fn_name, $skip_amount, $($input, $result,)*);
        );
    }

    #[test]
    fn position() {
        test_lexer!(position, 7, [
            "1234567" => &Position::new(1, 7),
        ]);
    }

    #[test]
    fn position_newline() {
        test_lexer!(position, 4, [
            "\r\n\r\n" => &Position::new(2, 0),
            "\n\n\r\n" => &Position::new(3, 0),
            "\n\r\n\r" => &Position::new(3, 0),
            "\n\n\n\n" => &Position::new(4, 0),
            "\r\r\r\r" => &Position::new(4, 0),
        ]);
    }

    #[test]
    fn read_octal() {
        test_lexer!(read_octal, [
            "0123456789_aaasdsad" => some "01234567",
            "567422505329asd" => some "56742250532",
        ]);
    }

    #[test]
    fn read_decimal() {
        test_lexer!(read_decimal, [
            "0123456789__asd_123-123" => some "0123456789",
        ]);
    }

    #[test]
    fn read_hexadecimal() {
        test_lexer!(read_hexadecimal, [
            "0123AB4567EF89D__asd_123-123" => some "0123AB4567EF89D",
        ]);
    }

    #[test]
    fn read_string() {
        test_lexer!(read_string, 6, [
            r#"voici "une longue chaîne de caractères valide"<-FIN"#
                => ok r#""une longue chaîne de caractères valide""#,
        ]);
    }

    #[test]
    fn read_string_escaped() {
        test_lexer!(read_string, [
            r#""longue chaîne doublement \" échappé \\"<-FIN"#
                => ok r#""longue chaîne doublement \" échappé \\""#,
        ]);
    }

    #[test]
    fn read_string_newline() {
        test_lexer!(read_string, [
            "\"ouah j'ai un retour à la ligne juste ici ->\n\"<-FIN"
                => ok "\"ouah j'ai un retour à la ligne juste ici ->\n\"",
        ]);
    }

    #[test]
    fn read_identifier() {
        test_lexer!(read_identifier, [
            "allo-ne me lit pas" => some "allo",
        ]);
    }

    #[test]
    fn read_identifier_with_question_mark() {
        test_lexer!(read_identifier, [
            "test? allo" => some "test?",
        ]);
    }

    #[test]
    fn skip_while() {
        let input = "je-veux      sauter-ces-espaces-blancs";
        let mut lexer = Lexer::new(input);
        for _ in 0..=6 {
            lexer.read();
        }

        // le `next()` caractère devrait être l'espace après le 'x'
        lexer.skip_while(|x| x.is_whitespace());

        assert_eq!(Some('s'), lexer.read());
    }

    #[test]
    fn take_while_number() {
        let input = "je-veux-123456-ces-nombres-seulement";
        let mut lexer = Lexer::new(input);
        for _ in 0..=7 {
            lexer.read();
        }

        // le `next()` caractère devrait être le '1' après le '-'
        let expected = Some("123456".to_string());
        let result = lexer.take_while(is_digit);

        assert_eq!(expected, result);
    }

    #[test]
    fn take_while_everything() {
        let input = "je-veux-tout-jusqu'à ce point.    ok";
        let mut lexer = Lexer::new(input);

        let expected = Some("je-veux-tout-jusqu'à ce point".to_string());
        let result = lexer.take_while(|ch| *ch != '.');

        assert_eq!(expected, result);
    }
}
