# Langage NB

Ceci est mon projet final de CEGEP incomplet.

Il n'y a pas de main, seulement des tests unitaires.
Seulement le parseur du langage a été créé, le compilateur n'ayant
jamais été implémenté/complêté à l'époque du projet dû à des problèmes
d'intégration avec LLVM dans le projet.

Le but était d'utiliser LLVM comme backend et de simplement écrire un "frontend"
(dans le _lingo_ de LLVM) pour le langage de programmation que je souhaitais créer.

## Description

NB est un langage de programmation procédural qui permet l'utilisation de
fonctions, conditions, boucles, chaînes de caractères et bien plus.

Le langage s’inspire de [Rust](https://www.rust-lang.org/),
[Elixir](https://elixir-lang.org/),
[ECMAScript](https://www.ecma-international.org/publications/standards/Ecma-262.htm),
[Haskell](https://www.haskell.org/) et bien d’autres.

## Travailler sur le projet

### FIXME

### Configurer l'environnement de développement Windows

### Configurer l'environnement de développement Linux

### Formattage

Ce projet utilise un outil pour formatter accordément aux conventions.

Utiliser l'utilitaire [rustfmt](https://github.com/rust-lang-nursery/rustfmt)
pour formatter le code.
Le fonctionnement est simple:
`cargo +nightly fmt`

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
