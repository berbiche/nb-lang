[![pipeline status](https://gitlab.com/nb-programming-language/parser/badges/parser/build.svg)](https://gitlab.com/nb-programming-language/parser/)
[![coverage report](https://gitlab.com/nb-programming-language/parser/badges/parser/coverage.svg)](https://gitlab.com/nb-programming-language/parser/commits/parser)
# Langage NB

## Description
NB est un langage de programmation procédural qui permet l'utilisation de
fonctions, conditions, boucles, chaînes de caractères et bien plus.

Le langage s’inspire de [Rust](https://www.rust-lang.org/),
[Elixir](https://elixir-lang.org/),
[ECMAScript](https://www.ecma-international.org/publications/standards/Ecma-262.htm),
[Haskell](https://www.haskell.org/) et bien d’autres.

## Travailler sur le projet

### Installation
1. Installer les outils de développement pour le langage de programmation [Rust](https://rustup.rs)
(__version >= 1.24__)
2. Installer un éditeur de texte qui supporte Rust:
    - [IntelliJ Idea Community](https://www.jetbrains.com/idea/download/)
    - [IntelliJ Rust](https://intellij-rust.github.io/)
3. Installer la version _nightly_ de _Rust_
    ```shell
    > rustup update
    > rustup install nightly
    ```

4. Changer la version par défaut utilisé de _Rust_ à _nightly_
    ```shell
    > rustup default nightly
    ```
    
5. (Optionnel) Installer l'utilitaire de formattage _rustfmt_
    ```shell
    > rustup component add rustfmt-preview
    ```

### Développement
Lorsque vous êtes prêt à compiler:
```shell
> cargo build
```

Lorsque vous êtes prêt à tester:
```shell
> cargo test --all
```

Pour générer la documentation (celle-ci se trouvera par défaut dans le dossier
`./target/doc`):
```shell
> cargo doc --release --no-deps -p 'nb' -p 'nb-parser' -p 'nb-std' --all-features
```
__À noter__:
- `cargo doc` ne génère (pour l'instant) pas la documentation de code qui n'est
pas `pub`lic
- La documentation des dépendances ne sera pas incluse (enlever `--no-deps`)


<!--### FIXME-->
<!--### Configurer l'environnement de développement Windows-->
<!--### Configurer l'environnement de développement Linux-->
### Formattage
Ce projet utilise un outil pour formatter le code accordément aux conventions.

Utiliser l'utilitaire [rustfmt](https://github.com/rust-lang-nursery/rustfmt)
pour formatter le code.
Le fonctionnement est simple:
```shell
> cargo +nightly fmt
```

## Mes motivations
Je me questionne parfois sur le procédé de compilation des langages de
programmation.

La conception d'un langage de programmation va donc me permettre de comprendre
le fonctionnement et le processus de compilation d’un programme.

Je cherche aussi à approfondir ma connaissance et avoir une compréhension que je
n’ai actuellement pas du fonctionnement des langages de programmation.

> avoir une compréhension que je n'ai actuellement pas

Je n'entends pas tout comprendre sur les langages de programmation par cette
phrase. Je désire seulement être plus conscient de mon ignorance en entreprenant
ce projet.

Par Nicolas Berbiche:
[Github](https://github.com/berbiche), [Gitlab](https://gitlab.com/berbiche)
