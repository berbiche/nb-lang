use token::{PositionOrSpan, Token};

use std::result;

/// Un type spécialisé pour les erreurs du lexer
pub type LResult<T> = result::Result<T, Error>;

// FIXME(Nicolas): Me remplir d'encore plus d'erreurs
#[derive(Debug, Eq, Fail, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Identifiant invalide
    #[fail(display = "Identifiant invalide: '{}' à {}", 0, 1)]
    InvalidIdentifier(String, PositionOrSpan),
    /// Une chaîne de caractère invalide dans l'entrée
    #[fail(display = "Chaîne de caractères invalide: '{}' à {}", 0, 1)]
    InvalidString(String, PositionOrSpan),
    /// Début de chaîne de caractères manquant '"'
    #[fail(display = "Début de chaîne de caractères manquant à {}", 0)]
    MissingStringBeginning(PositionOrSpan),
    /// End-of-file atteint avant la fin de l'opération désiré
    #[fail(display = "End-of-File atteint avant la fin de la séquence désiré à {}", 0)]
    UnexpectedEOF(PositionOrSpan),
    /// Le lexer s'attendait à un certain symbol, mais il en a rencontré un autre
    #[fail(display = "Caractère inattendu: '{}' à {}", 0, 1)]
    UnexpectedCharacter(String, PositionOrSpan),
    #[fail(display = "Symbole inattendu: '{}' à {}", 0, 1)]
    UnexpectedToken(Token, PositionOrSpan),
    /// Chaîne de caractères non-terminée, peut-être dû à un EOF comme autre chose
    #[fail(display = "Chaîne de caractères n'est pas terminée à {}", 0)]
    UnterminatedString(PositionOrSpan),
}
