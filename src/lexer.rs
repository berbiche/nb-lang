//! Fonctions pour la phase d'analyse lexicale
use std::result::Result;
use std::iter::Peekable;
use std::str::Chars;
use std::borrow::Cow;

// FIXME: Me remplir d'erreurs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
}

/// Le Lexer est un wrapper sur un itérateur qui lit les caractères pour former des
/// lexèmes.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// l'entrée à parse, une séquence de caractères itérable
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// Construit un Lexer depuis une chaîne de caractères
    pub fn new(input: &'a str) -> Self {
        assert!(
            input.len() > 0,
            "La chaîne de caractères ne peut être vide"
        );

        Lexer {
            input: input.chars().peekable(),
        }
    }

    /// Permet de voir le prochain caractère sans consommer le caractère
    #[inline]
    pub fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// Retourne le prochain caractère, le consommant du coup
    #[inline]
    pub fn read(&mut self) -> Option<char> {
        self.input.next()
    }

    /// Lit un identifiant et le retourne
    pub fn read_identifier(&mut self) -> Option<String> {
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
    pub fn read_decimal(&mut self) -> Option<String> {
        self.take_while(is_digit)
    }

    /// Permet de lire des nombres hexadécimaux
    pub fn read_hexadecimal(&mut self) -> Option<String> {
        self.take_while(is_hex)
    }

    /// Permet de lire des nombres octaux
    pub fn read_octal(&mut self) -> Option<String> {
        self.take_while(is_octal)
    }

    /// Lit une chaîne de caractères jusqu'à un '"' non-échappé
    pub fn read_string(&mut self) -> Option<String> {
        // nous voulons itérer sur la séquence jusqu'à ce que nous trouvions
        // le caractère '"' qui n'a pas le caractère d'échappe '\\' avant
        // et que ce caractère d'échappe n'est pas échappé
        let mut st = String::new();

        // on prend d'abord le premier '"'
        match self.read() {
            Some(ch) if ch == '"' => st.push(ch),
            _ => return None,
        };

        // nous devons connaître le caractère précédent pour savoir si échappé
        let mut previous_ch = '\0';
        while let Some(current_ch) = self.read() {
            if is_newline(&current_ch) {
                break;
            }

            st.push(current_ch);

            // si l'échappe est échappé
            if previous_ch == '\\' && current_ch == '\\' {
                previous_ch = '\0';
                continue;
            }

            if previous_ch != '\\' && current_ch == '"' {
                break;
            }

            previous_ch = current_ch;
        }
        Some(st)
    }

    /// Saute les espaces-blancs, incluant le retour à la ligne
    pub fn skip_whitespace(&mut self) {
        self.skip_while(|ch| ch.is_whitespace());
    }

    /// Notre version de skip_while qui ne consomme pas le caractère
    /// si celui-ci ne respect pas le prédicat
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
    /// si celui-ci ne respect pas le prédicat
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

#[inline]
fn is_digit(ch: &char) -> bool {
    ch.is_digit(10)
}

#[inline]
fn is_hex(ch: &char) -> bool {
    ch.is_digit(16)
}

#[inline]
fn is_octal(ch: &char) -> bool {
    ch.is_digit(8)
}

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

    macro_rules! test_lexer {
        (@decl $lexer:ident, $input:expr) => (let mut $lexer = Lexer::new($input););

        (@assert $fn_name:ident, $lexer:ident, $result:expr) => (
            assert_eq!(Some($result.to_string()), $lexer.$fn_name());
        );

        (@do $lexer:ident, $skip_amount:expr) => (
            for _ in 0..$skip_amount {
                $lexer.read();
            }
        );

        // test_lexer!(fn_name, ["" => ""])
        ($fn_name:ident, [
            $( $input:expr => $result:expr ),+
        ]) => (
            $(
                test_lexer!(@decl lexer, $input);
                test_lexer!(@assert $fn_name, lexer, $result);
            )*
        );

        // test_lexer!(fn_name, [6; "" => ""]
        ($fn_name:ident, $skip_amount:expr, [
            $(
                $input:expr => $result:expr
            ),+
        ]) => (
            $(
                test_lexer!(@decl lexer, $input);
                test_lexer!(@do lexer, $skip_amount);
                test_lexer!(@assert $fn_name, lexer, $result);
            )*
        );

        // test_lexer!(fn_name, []
//        ($fn_name:ident, [
//            $(
//                [ s $skip_amount:expr; $input:expr => $result:expr ]
//            ),+
//        ]) => (
//            $(
//                test_lexer!(@decl lexer, $input);
//                lexer.skip_while($skip_while);
//                test_lexer!(@assert $fn_name, lexer, $result);
//            )*
//        );
    }

    #[test]
    fn read_octal() {
        test_lexer!(read_octal, [
            "0123456789_aaasdsad" => "01234567",
            "567422505329asd" => "56742250532"
        ]);
    }

    #[test]
    fn read_decimal() {
        test_lexer!(read_decimal, [
            "0123456789__asd_123-123" => "0123456789"
        ]);
    }

    #[test]
    fn read_hexadecimal() {
        test_lexer!(read_hexadecimal, [
            "0123AB4567EF89D__asd_123-123" => "0123AB4567EF89D"
        ]);
    }

    #[test]
    fn read_string() {
        test_lexer!(read_string, 6, [
            r#"voici "une longue chaîne de caractères valide"<-FIN"#
                => r#""une longue chaîne de caractères valide""#
        ]);
    }

    #[test]
    fn read_string_escaped() {
        test_lexer!(read_string, [
            r#""longue chaîne doublement \" échappé \\""<-FIN"#
                => r#""longue chaîne doublement \" échappé \\""#
        ]);
    }

    #[test]
    fn read_string_newline() {
        test_lexer!(read_string, [
            r#""ouah j'ai un retour à la ligne juste ici ->
            la chaîne de caractère en est à sa fin, comment le parseur va "handle" cela?"#
            => r#""ouah j'ai un retour à la ligne juste ici ->"#
        ]);
    }

    #[test]
    fn read_identifier() {
        test_lexer!(read_identifier, [
            "allo-ne me lit pas" => "allo"
        ]);
    }

    #[test]
    fn read_identifier_with_question_mark() {
        test_lexer!(read_identifier, [
            "test? allo" => "test?"
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
    fn take_while() {
        let input = "je-veux-123456-ces-nombres-seulement";
        let mut lexer = Lexer::new(input);
        for _ in 0..=7 {
            lexer.read();
        }

        // le `next()` caractère devrait être le '1' après le '-'
        let result = lexer.take_while(is_digit);

        assert_eq!(Some("123456".to_string()), result);
    }
}
