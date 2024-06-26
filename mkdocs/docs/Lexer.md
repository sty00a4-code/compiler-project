Der Lexer führt eine lexikalische Analyse des Input-Texts durch, wobei er Buchstaben in Symbolen, Schlüsselwörtern, oder Zahlen zusammenfasst. Diese nennt man Token (Englisch für "Zeichen" oder "Symbol"). Diese helfen uns später um eine bessere syntaktische Analyse mit dem Parser zu vollführen.
## Tokens
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
- `Ident`: Kurz für "identifier", ein einfaches Wort ohne jetzige Bedeutung
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
Als nächstes habe ich hier verschiedene Symbole definiert die später gut zu gebrauchen sind.
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
Zum Schluss habe ich hier noch ein paar Schlüsselwörter definiert.
### Position
Um die Tokens später für die Fehlerbehandlung lokalisieren zu können im Text habe ich zwei Spezielle Datentypen erstellt, die ich später dauernd benutzen werde.
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
`Located<T>` ist eine generalisierte Struktur, die einen Wert (`value`) eines beliebigen Datentypen (`T`) und eine Position (`pos`) zusammen speichert. Um die Tokens zu Lokalisieren benutze ich oft die Variante `Located<Token>`, was bedeutet, dass die schlussendliche Struktur dafür wie folgt aussieht:
```rust
struct Located<Token> {
	value: Token,
	pos: Position
}
```
`T` wird hier zu `Token`. `Located<T>` ist also keine eigene Struktur, sondern ist lediglich eine Vorlage für eine echte Struktur.
## Lexer
Der Lexer wird nun wie folgt implementiert:
```rust
struct Lexer<'a> {
	chars: Peekable<Chars<'a>>, // Iterator über einen Text
	ln: usize, // Zeilennummer
	col: usize, // Spaltennummer
}
```
Die Struktur Lexer besteht aus dem Feld `chars`, welche ein Iterator über `char` ist, und einer Zeilennummer (`ln`) sowie einer Spaltennummer (`col`). Das sind alle Daten, die man benötigen um erfolgreich Tokens zu generieren.
```rust
impl<'a> Iterator for Lexer<'a> {
	type Item = char;
	fn next(&mut self) -> Option<Self::Item> {
		let c = self.chars.next()?; // nächster Buchstabe
		if c == '\n' { // neue Zeile
			self.ln += 1;
			self.col = 0;
		} else {
			self.col += 1;
		}
		Some(c) // gibt den Buchtstaben wider
	}
}
```
Um noch mehr Funktionalitäten für den Lexer zu implementieren, benutzen wir den `Iterator` Trait in Rust. Dieser benötigt nur eine Funktion `next` und den Typen des Widergabewertes `Item` was in unserem Fall `char` ist. Rust erstellt dann automatisch noch mehr Funktionen für den Lexer, da diese nur die `next` Funktion benötigen.
```rust
trait Lexable: Sized {
	type Error;
	
	fn lex_next<'a>(lexer: &mut Lexer<'a>) -> Result<Option<Located<Self>>, Located<Self::Error>>;
	
	fn lex(text: &str) -> Result<Vec<Located<Self>>, Located<Self::Error>> {
		let mut lexer = Lexer {
			chars: text.chars().peekable(),
			ln: 0,
			col: 0
		};
		let mut tokens = vec![];
		while let Some(token) = Self::lex_next(&mut lexer)? {
			tokens.push(token)
		}
		Ok(tokens)
	}
}
```
Hier definiere ich einen eigenen Trait namens `Lexable`. Das einzige was hier wichtig ist zu verstehen ist, dass Traits für jeglichen Datentypen Implementiert werden können mit `impl Trait for Type { ... }` , und dass dieser `Lexable` Trait den Implementierer zwingt die `lex_next` Funktion und den `Error` Typen zu definieren.
```rust
enum LexError {
	BadCharacter(char),
	ParseNumberError(ParseFloatError),
	UnclosedString,
}
impl Lexable for Token {
	type Error = LexError;
	fn lex_next<'a>(lexer: &mut Lexer<'a>) -> Result<Option<Located<Self>>, Located<Self::Error>> {
		...
	}
}
```
Genau das mache ich hier für `Token`. Dort wo die drei Punkte sind werde ich in folge die [[#Schritte des Lexers]]. `LexError` ist ein `enum` um alle Fehler die entstehen können beim Lexen darzustellen.
### Schritte des Lexers
#### Leerzeichen
```rust
while let Some(c) = lexer.peek() {
	if !c.is_ascii_whitespace() {
		break;
	}
	lexer.next();
}
let pos = lexer.pos();
let Some(c) = lexer.next() else {
	return Ok(None);
};
...
```
Als erstes ist es wichtig alle Leerzeichen (dazu zählen auch Zeilenumbrüche) zu überspringen, damit diese beliebig zwischen jedem Token stehen können. Dafür ist die `while let` Schleife da. Danach merkt sicher der Lexer die Position un den ersten Buchstaben des zu lexendem Tokens.
#### Symbole
```rust
match c {
	'=' => {
		if lexer.peek() == Some(&'=') {
			lexer.next();
			Ok(Some(Located::new(Token::EqualEqual, pos)))
		} else {
			Ok(Some(Located::new(Token::Equal, pos)))
		}
	}
	',' => Ok(Some(Located::new(Token::Comma, pos))),
	'.' => Ok(Some(Located::new(Token::Dot, pos))),
	'(' => Ok(Some(Located::new(Token::ParanLeft, pos))),
	')' => Ok(Some(Located::new(Token::ParanRight, pos))),
	'[' => Ok(Some(Located::new(Token::BracketLeft, pos))),
	']' => Ok(Some(Located::new(Token::BracketRight, pos))),
	'{' => Ok(Some(Located::new(Token::BraceLeft, pos))),
	'}' => Ok(Some(Located::new(Token::BraceRight, pos))),
	'+' => Ok(Some(Located::new(Token::Plus, pos))),
	'-' => Ok(Some(Located::new(Token::Minus, pos))),
	'*' => Ok(Some(Located::new(Token::Star, pos))),
	'/' => Ok(Some(Located::new(Token::Slash, pos))),
	'%' => Ok(Some(Located::new(Token::Percent, pos))),
	'^' => Ok(Some(Located::new(Token::Exponent, pos))),
	'<' => {
		if lexer.peek() == Some(&'=') {
			lexer.next();
			Ok(Some(Located::new(Token::LessEqual, pos)))
		} else {
			Ok(Some(Located::new(Token::Less, pos)))
		}
	}
	'>' => {
		if lexer.peek() == Some(&'=') {
			lexer.next();
			Ok(Some(Located::new(Token::GreaterEqual, pos)))
		} else {
			Ok(Some(Located::new(Token::Greater, pos)))
		}
	}
	'&' => Ok(Some(Located::new(Token::Ampersand, pos))),
	'|' => Ok(Some(Located::new(Token::Pipe, pos))),
	'!' => {
		if lexer.peek() == Some(&'=') {
			lexer.next();
			Ok(Some(Located::new(Token::ExclamationEqual, pos)))
		} else {
			Ok(Some(Located::new(Token::Exclamation, pos)))
		}
	}
	...
}
```
Als erstes wird geschaut ob der Buchstabe ein bestimmtes Zeichen ist was für die Sprache von Bedeutung ist wie `=` oder `+` usw. Falls das der Fall ist geben wir einfach das Symbol als Token wider.
Wie man sehen kann benutze ich dafür den `match` Zweig, der in Rust ein sogenanntes Pattern-Matching (Mustervergleichen) benutzt um die Art des Buchstabens zu ermitteln. Es ist ein sehr nützlicher Zweig, den ich immer wieder benutze, auch später.
#### Strings
```rust
match c {
	...
	'"' => {
		let mut string = String::new();
		while let Some(c) = lexer.peek() {
			if *c == '"' {
				break;
			}
			string.push(lexer.next().unwrap())
		}
		if lexer.next() != Some('"') {
			return Err(Located::new(LexError::UnclosedString, pos))
		}
		Ok(Some(Located::new(Token::String(string), pos)))
	}
	...
}
```
Ein spezielles Symbol für die Sprache ist das Anführungszeichen, denn zwei von diesen umgeben immer einen String. Das heißt wenn der Lexer eines begegnet, sammelt er alle Buchstaben bis er das nächste Anführungszeichen hat, und dann gibt er diesen String als Token wider.
#### Zahlen
```rust
match c {
	...
	c if c.is_ascii_digit() => {
		let mut number = String::from(c);
		while let Some(c) = lexer.peek() {
			if !c.is_ascii_digit() {
				break;
			}
			number.push(lexer.next().unwrap())
		}
		if lexer.peek() == Some(&'.') {
			number.push(lexer.next().unwrap());
			while let Some(c) = lexer.peek() {
				if !c.is_ascii_digit() {
					break;
				}
				number.push(lexer.next().unwrap())
			}
		}
		Ok(Some(Located::new(
			Token::Number(
				number
					.parse()
					.map_err(LexError::ParseNumberError)
					.map_err(|err| Located::new(err, pos.clone()))?,
			),
			pos,
		)))
	}
	...
}
```
Zahlen werden auf diese Weise gefunden. Wenn der Buchstabe eine ASCII Nummer ist werden alle darauf folgenden ASCII Nummern erst in einem String gesammelt, der dann am Ende in eine Dezimalzahl umgewandelt wird. Falls der Lexer einem Punkt begegnet nimmt er diesen mit und sammelt noch alle Nummern die danach kommen auf.
#### Identifiers
```rust
match c {
	...
	c if c.is_alphanumeric() || c == '_' => {
		let mut ident = String::from(c);
		while let Some(c) = lexer.peek() {
			if !c.is_alphanumeric() && *c != '_' {
				break;
			}
			ident.push(lexer.next().unwrap())
		}
		Ok(Some(Located::new(Token::ident(ident), pos)))
	}
	...
}
```
Identifier werden auf die gleiche Art wie Zahlen aufgenommen, nur kann es hier keinen Punkt geben, was es noch einfacherer macht. Teile eines Identifiers sind alle Alpha-nummerische Buchstaben sowie Unterstriche.
```rust
impl Token {
	...
	fn ident(ident: String) -> Self {
		match ident.as_str() {
			"let" => Self::Let,
			"def" => Self::Def,
			"if" => Self::If,
			"else" => Self::Else,
			"while" => Self::While,
			"for" => Self::For,
			_ => Self::Ident(ident),
		}
	}
	...
}
```
Falls der Identifier einen der Schlüsselwörter entspricht, wird das Schlüsselwort wider gegeben und nicht der eigentliche Identifier.
#### Errors
```rust
match c {
	...
	c => Err(Located::new(LexError::BadCharacter(c), pos)),
}
```
Wenn keiner der vorherigen Muster gepasst hat, gibt der Lexer einen Fehler wider, der besagt, dass ein nicht erkennbarer Buchstabe in dem Input ist, und schließt somit das Programm.
#### Sammeln
```rust
fn lex(text: &str) -> Result<Vec<Located<Token>>, Located<LexError>> {
	let mut lexer = Lexer {
		chars: text.chars().peekable(),
		ln: 0,
		col: 0
	};
	let mut tokens = vec![];
	while let Some(token) = Token::lex_next(&mut lexer)? {
		tokens.push(token)
	}
	Ok(tokens)
}
```
Diese Funktion führt die Beschrieben Schritte immer wieder aus bis es am Ende des Inputs angekommen ist. Somit erhalten wir am Ende eine Liste mit Tokens und können nun weiter mit dem Parsing machen.

Weiter zum [Parser](Parser.md).