Ein klassischer Compiler kann nicht einfach vom Text zum Byte Code gehen, sondern muss vorher zu verschiedene Strukturen analysiert werden. Die meisten Compiler durchlaufen dafür folgenden Schritte, die jeweils auf dem vorherigen Schritt aufbauen:
1. __Lexikalische Analyse__: Die Atomaren Teile des Textes (sogenannte Tokens)
2. __Syntaktische Analyse__: Der Syntax Baum, der darstellt wo was ist.
3. __Compilierung__: Die tatsächliche Umwandlung in den Byte Code.

Weiter zum [Lexer](Lexer.md)