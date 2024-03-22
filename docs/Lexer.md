Der Lexer führt eine lexikalische Analyse des Input-Texts durch, wobei er Buchstaben in Symbolen, Schlüsselwörtern, oder Zahlen zusammenfasst. Diese nennt man Token (Englisch für "Zeichen" oder "Symbol"). Diese helfen uns später um eine bessere syntaktische Analyse mit dem Parser zu vollführen.

Um Tokens repräsentieren zu können benutze ich in Rust ein sogenanntes `enum`. Dieser Typ von Datenstruktur erlaubt es einem seinen eigenen Datentypen zu erstellen, der beliebig viele Varianten haben kann.
```rust
pub enum Token {
	// word kinds
	Ident(String),
	Number(f64),
	String(String),
	
	...
}
```
Hier habe ich nun die ersten Varianten eines Tokens in meiner Sprache definiert:
- `Ident`: Kurz für "identifier", ein einfaches Wort ohne Bedeutung
- `Number`: Eine Dezimalzahl (`f64`: floating point number mit einer Bit-Größe von 64)
- `String`: Eine Buchstaben folge von beliebiger Größe
```rust
pub enum Token {
	...
	
	// symbols
	Equal, // =
	Comma, // ,
	Dot, // .
	ParanLeft, // (
	ParanRight, // )
	BracketLeft, // [
	BracketRight, // ]
	BraceLeft, // {
	BraceRight, // }
	Plus, // +
	Minus, // -
	Star, // *
	Slash, // /
	Percent, // %
	Exponent, // ^
	EqualEqual, // ==
	ExclamationEqual, // !=
	Less, // <
	Greater, // >
	LessEqual, // <=
	GreaterEqual, // >=
	Ampersand, // &
	Pipe, // |
	Exclamation, // !
	
	...
}
```
Als nächstes habe ich verschiedene Symbole definiert die später gut zu gebrauchen sind.
```rust
pub enum Token {
	...
	
	// key words
	Let,
	Def,
	If,
	Else,
	While,
	For,
}
```
Zum Schluss habe ich noch ein paar Schlüsselwörter definiert.

Eine Sache auf die ich nicht eingegangen bin war, dass ich einen Datentyp für Lokalisierung von Tokens geschrieben habe, weil es jetzt nicht so wichtig war. Dennoch hier eine kurze Erklärung was diese genau machen, da ich sie später dauernd benutzen werde.
```rust
struct Position {
	ln: usize,
	col: usize,
}
struct Located<T> {
	value: T,
	pos: Position,
}
```
`Position` ist eine Struktur, die die Zeilen- und Spaltennummer sich merkt in Form von zwei `usize` (eine ganze Zahl die nicht negativ sein kann).
`Located<T>` ist eine generalisierte Struktur, die einen Wert (`value`) eines beliebigen Datentypen (`T`) und eine Position (`pos`) zusammen speichert. Um die Tokens zu Lokalisieren benutze ich oft die Variante `Located<Token>`, was bedeutet das die schlussendliche Struktur wie folgt aussieht:
```rust
struct Located<Token> {
	value: Token,
	pos: Position
}
```
