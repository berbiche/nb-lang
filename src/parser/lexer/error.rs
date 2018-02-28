use token::{Keyword, PositionOrSpan, Token, TokenType};

use std::result;

/// Un type spécialisé pour les erreurs du lexer
pub type LResult<T> = result::Result<T, Error>;

// FIXME(Nicolas): Me remplir d'encore plus d'erreurs
#[derive(Debug, Eq, Fail, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Le `Token` attendu n'est pas celui présent
    #[fail(display = "Symbole attendu: '{0}' plutôt que '{1}' à {2}", 0, 1, 2)]
    ExpectedToken(String, TokenType, PositionOrSpan),
    /// Identifiant invalide
    #[fail(display = "Identifiant invalide: '{0}' à {1}", 0, 1)]
    InvalidIdentifier(String, PositionOrSpan),
    /// Nombre invalide
    #[fail(display = "Nombre invalide: '{0}' à {1}", 0, 1)]
    InvalidNumber(String, PositionOrSpan),
    /// Une chaîne de caractère invalide dans l'entrée
    #[fail(display = "Chaîne de caractères invalide: '{0}' à {1}", 0, 1)]
    InvalidString(String, PositionOrSpan),
    /// Début de chaîne de caractères manquant '"'
    #[fail(display = "Début de chaîne de caractères manquant à {0}", 0)]
    MissingStringBeginning(PositionOrSpan),
    /// Utilisation d'un mot-clé réservé par le langage
    #[fail(display = "Mot-clé réservé: '{0}' à {1}", 0, 1)]
    ReservedKeyword(TokenType, PositionOrSpan),
    /// Le lexer s'attendait à un certain symbol, mais il en a rencontré un autre
    #[fail(display = "Caractère inattendu: '{0}' à {1}", 0, 1)]
    UnexpectedCharacter(String, PositionOrSpan),
    /// End-of-file atteint avant la fin de l'opération désiré
    #[fail(display = "End-of-File atteint avant la fin de la séquence désiré à {0}", 0)]
    UnexpectedEOF(PositionOrSpan),
    /// Chaîne de caractères non-terminée, peut-être dû à un EOF comme autre chose
    #[fail(display = "Chaîne de caractères non terminée à {0}", 0)]
    UnterminatedString(PositionOrSpan),
    /// Un `Token` innatendu a été rencontré
    #[fail(display = "Symbole inattendu: '{0}' à {1}", 0, 1)]
    UnexpectedToken(TokenType, PositionOrSpan),
}
