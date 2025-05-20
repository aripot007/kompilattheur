# Rapport PCL2
## Table des symboles
La structure de la table des symboles suit ce schéma :

|clé|nom|symbole|type|offset(octet)
|--|--|--|--|--|
1|f|Function|Int|-
2|i|Parameter|Int|-16
3|foo|Variable|List|0
4|toto|Variable|String|+16

Les données sont enregistrées dans une Hashmap, avec en clé un entier représentatif de la variable, donné par la table des identifier.
Les types que nous avons définis sont : 
- None 
- Bool
- Int
- String
- List
- Range
- Any
- Weak

Chaque type est représenté sur 16 octets, dont 8 pour stocker la donnée et 8 pour stocker le type.
Cela nous permet d'utiliser des types Any et Weak, afin de typer dynamiquement nos variables. 
 <!-- TODO : Expliquer comment marche nos types Weak et Any en détail, je suis pas à l'aise sur la notion -->

## Schéma de traduction
<!-- TODO: là je peux pas vraiment aider 💀, ce que j'ai fait est basique : "Il faut présenter les chémas de traduction (source smollpp vers LLVM-IR pour les structures intéréssantes : appel fonction, conditionnelles imbriquées, appels récursifs ?, ...)"-->

## Gestion de Projet :
### Aristide
**Typage** : typage statique (); typage dynamique ()
**Controle sémantique statiques** : verification des types statiques (); 
**Assembleur** : structure base (); hello world POC (); Variables (); Pointeurs (); Strings (); Lists (); 

### Baptiste
**Controles sémantiques statiques** : verification des acces de liste ();
**Config LLVM** : configuration des librairies et recherches (); config du linker LLVM pour Linux (); 
**Assembleur** : expression (); loops (); "and", "or", "not" (); 

### Luca 
**TDS** : remplissage de la TDS depuis l'AST (10h); gestion des offsets (5h)
**Contrôle sémantiques statiques** : portée des variables (5h); 
**Configuration LLVM** : adaptation config linker LLVM pour macOS en préservant config Linux (4h)
**Assembleur** : opérations binaire (4h); compairaison statiques (4h); fonctions internes : range et list (4h)

### Romain
**TDS** : affinage des structures de données ();
**Contrôles sémantiques statiques** : verification des arguments et retours de fonctions (); 
**Assembleur** :  typage dynamique des fonctions (); branching if else (); comparaison generique ();