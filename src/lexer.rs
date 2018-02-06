// use token::*;

/// Le Lexer est une sorte d'itérateur qui lit les caractères pour former des
/// lexèmes.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// la chaîne de caractères
    input: &'a str,
    /// position actuelle dans la chaîne
    position: &'a str,
    /// prochaine position dans la séquence lookahead(1)
    next_position: &'a str,
    /// caractère utf-8
    character: u8,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        assert!(input.len > 0, "La chaîne de caractères ne peut être vide");
        Lexer {
            input,
            position: input[0],
            next_position: 0,
            character: 0
        }
    }

    fn peak_character(&mut self) -> &str {
        unimplemented!()
    }

    fn read_character(&mut self) -> &str {
        unimplemented!()
    }
}
