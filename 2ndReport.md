# Rapport PCL2
## Table des symboles
La structure de la table des symboles suit ce schéma :

|clé|nom|symbole|type|offset(octet)
|--|--|--|--|--|
1|f|Function|Int|-
2|i|Parameter|Int|-9
3|foo|Variable|List|0
4|toto|Variable|String|+9

Les données sont enregistrées dans une Hashmap, avec en clé un entier représentatif de la variable, donné par la table des identifier.
Les types de symboles sont : 
- Paramètre
- Fonction
- Variable

Les fonctions sont typées par leur type de retour, les paramètres et variables sont définies selon les types : 
- None 
- Bool
- Int
- String
- List
- Any
- Weak
- Range (Itérateur pour la fonction range)

Chaque type est représenté sur 9 octets, dont 8 pour stocker la donnée et 1 pour stocker le type.
Cela nous permet d'utiliser des types `Any` et `Weak`, afin de typer dynamiquement nos variables. 
Le type `Any` permet de representer n'importe quel type, il est nécéssaire pour le contenu des listes. Un acces à une liste donne un type `Any`, les controles sémantiques seront fait dynamiquement plus tard à l'éxécution.

Le type `Weak` permet de faire soit une intersection de types ou une union de types afin de pouvoir inférer un type à la compilation. Ex: deux variables faisant une opération "-" n'est possible qu'entre deux `Int`, le type final sera un `Int`. Si une fonction retourne soit un booléen soit un string `Weak(String, Bool)`, alors une opération "-" avec le retour de cette fonction donnera une erreur statique.
Le type `Range` est un type interne. Il contient un `Int` et imite le fonctionnement d'un itérateur Python dans une boucle `for`et une fonction `list`.

## Schéma de traduction
```Python
def fibonacci(n): # 🟥
    if n <= 0: # 0️⃣
        return 0 # 🔺
    if n == 1: # 1️⃣
        return 1 # 🔻
    return fibonacci(n-1) + fibonacci(n-2)

fibonacci(5) # 🟩
```

```LLVM
define i32 @main() {
entry:
  %function_call_fibonacci = call %dynamic_type_struct @__smolpp_user_f_fibonacci(%dynamic_type_struct { i8 4, i64 5 }) ; 🟩
  ret i32 0
}

define %dynamic_type_struct @__smolpp_user_f_fibonacci(%dynamic_type_struct %0) { ; 🟥
function_entry:
  %alloca_param_n = alloca %dynamic_type_struct, align 8
  store %dynamic_type_struct %0, ptr %alloca_param_n, align 4
  %load_n = load %dynamic_type_struct, ptr %alloca_param_n, align 4
  %value_field = extractvalue %dynamic_type_struct %load_n, 1
  %lte = icmp sle i64 %value_field, 0
  %int_cast = sext i1 %lte to i64
  %with_value = insertvalue %dynamic_type_struct { i8 2, i64 undef }, i64 %int_cast, 1
  %value_field1 = extractvalue %dynamic_type_struct %with_value, 1
  %bool_if = trunc i64 %value_field1 to i1
  br i1 %bool_if, label %then, label %else ; 0️⃣

then:                                             ; preds = %function_entry
  ret %dynamic_type_struct { i8 4, i64 0 } ; 🔺

else:                                             ; preds = %function_entry
  br label %merge

merge:                                            ; preds = %else
  %load_n2 = load %dynamic_type_struct, ptr %alloca_param_n, align 4
  %value_field3 = extractvalue %dynamic_type_struct %load_n2, 1
  %eq = icmp eq i64 %value_field3, 1
  %int_cast4 = sext i1 %eq to i64
  %with_value5 = insertvalue %dynamic_type_struct { i8 2, i64 undef }, i64 %int_cast4, 1
  %value_field9 = extractvalue %dynamic_type_struct %with_value5, 1
  %bool_if10 = trunc i64 %value_field9 to i1
  br i1 %bool_if10, label %then6, label %else7 ; 1️⃣

then6:                                            ; preds = %merge
  ret %dynamic_type_struct { i8 4, i64 1 } ; 🔻

else7:                                            ; preds = %merge
  br label %merge8

merge8:                                           ; preds = %else7
  %load_n11 = load %dynamic_type_struct, ptr %alloca_param_n, align 4
  %value_field12 = extractvalue %dynamic_type_struct %load_n11, 1
  %sub = sub i64 %value_field12, 1
  %with_value13 = insertvalue %dynamic_type_struct { i8 4, i64 undef }, i64 %sub, 1
  %function_call_fibonacci = call %dynamic_type_struct @__smolpp_user_f_fibonacci(%dynamic_type_struct %with_value13)
  %load_n14 = load %dynamic_type_struct, ptr %alloca_param_n, align 4
  %value_field15 = extractvalue %dynamic_type_struct %load_n14, 1
  %sub16 = sub i64 %value_field15, 2
  %with_value17 = insertvalue %dynamic_type_struct { i8 4, i64 undef }, i64 %sub16, 1
  %function_call_fibonacci18 = call %dynamic_type_struct @__smolpp_user_f_fibonacci(%dynamic_type_struct %with_value17)
  %value_field19 = extractvalue %dynamic_type_struct %function_call_fibonacci, 1
  %value_field20 = extractvalue %dynamic_type_struct %function_call_fibonacci18, 1
  %add = add i64 %value_field19, %value_field20 ; ➕
  %with_value21 = insertvalue %dynamic_type_struct { i8 4, i64 undef }, i64 %add, 1
  ret %dynamic_type_struct %with_value21
}
```


## Gestion de Projet :
### Aristide
**Typage** : typage statique (10h); typage dynamique (3h)
**Controle sémantique statiques** : verification des types statiques (5h); 
**Controle sémantique statiques** : assertion des types pour les opérations (3h);
**Assembleur** : print de base (1h); print générique (1h); Variables (4h); Strings (3h); Listes (5h); arithmétique générique (2h); type Weak (17h); boucle while (1h); fonctions de la librairie standard (input) (2h); concaténation des strings et des listes (5h); comparaison des strings (1h);

### Baptiste
**Controles sémantiques dynamique** : comparaison generique (10h); "and", "or", "not", "if" -> cast n'importe quel type en booléen (2h); verif boucle for : list ou range (4h)
**Config LLVM** : configuration des librairies et recherches, choix entre inkwell et llvm-ir (20h); config du linker LLVM pour Linux (20h); adaptation du CLI pour compilation et execution (5h)
**Assembleur** : structure base : function main, setup initial (4h); expression (2h); loops : boucle "for" (12h); fonctions "and", "or", "not" (4h); fonction "len", "list", "range" (6h); creation d'une librairie standard (3h); 

### Luca 
**TDS** : remplissage de la TDS depuis l'AST (7h); gestion des offsets (5h)
**Contrôle sémantiques statiques** : portée des variables (4h); 
**Configuration LLVM** : adaptation config linker LLVM pour macOS en préservant config Linux (4h)
**Assembleur** : opérations binaire (3h); compairaison statiques (4h); fonctions internes : range et list (5h); affichage des erreurs pendant l'éxecution (2h); localisation des erreurs d'execution (5h); 

### Romain
**TDS** : affinage des structures de données (2h);
**Contrôles sémantiques statiques** : verification des arguments et retours de fonctions (3h); 
**Assembleur** :  typage dynamique des fonctions (4h); appel de fonctions (2h);  branching if else (2h); accès et assignation de liste (3h); comparaison liste (6h); appels de fonction mutuellement récursif (1h); fonction interne int (3h); fonction interne surchage plus (4h)
