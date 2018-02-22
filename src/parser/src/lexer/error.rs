use lexer::Position;

use std::result;

/// Un type spécialisé pour les erreurs du lexer
pub type LResult<T> = result::Result<T, Error>;

// FIXME(Nicolas): Me remplir d'encore plus d'erreurs
#[derive(Debug, Eq, Fail, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Identifiant invalide
    #[fail(display = "Identifiant invalide: '{}' à {}", 0, 1)]
    InvalidIdentifier(String, Position),
    /// Une chaîne de caractère invalide dans l'entrée
    #[fail(display = "Chaîne de caractères invalide: '{}' à {}", 0, 1)]
    InvalidString(String, Position),
    /// Début de chaîne de caractères manquant '"'
    #[fail(display = "Début de chaîne de caractères manquant à {}", 0)]
    MissingStringBeginning(Position),
    /// End-of-file atteint avant la fin de l'opération désiré
    #[fail(display = "End-of-File atteint avant la fin de la séquence désiré à {}", 0)]
    UnexpectedEOF(Position),
    /// Le lexer s'attendait à un certain symbol, mais il en a rencontré un autre
    #[fail(display = "Caractère inattendu: '{}' plutôt que '{}' à {}", unexp, exp, pos)]
    UnexpectedSymbol {
        exp: char,
        unexp: char,
        pos: Position,
    },
    /// Chaîne de caractères non-terminée, peut-être dû à un EOF comme autre chose
    #[fail(display = "Chaîne de caractères n'est pas terminée à {}", 0)]
    UnterminatedString(Position),
}
