## Ziel
Das Ziel eines Parser ist es die vom lexer erstellten Token in eine Abstract-Syntax-Tree (abstrakter Syntaxbaum) umzuwandeln.
Der Input:
```lua
1 + 2 * 3
```
Die Tokens:
```lua
[NUMBER(1)] [PLUS] [NUMBER(2)] [STAR] [NUMBER(3)]
```
Der AST:
```h
	+
  /   \
 1     *
	  / \
	 2   3
```
In JSON dargestellt:
```json
{
	"operator": "+",
	"left": 1,
	"right": {
		"operator": "*",
		"left": 2,
		"right": 3
	}
}
```
## Parser Expression Grammar
Natürlich soll unser parser noch mehr Arten von ASTs erkennen können als nur Binäre Operationen. Zum Beispiel eine Variabel-Erstellung. Um Parser aufschreiben zu können benutzt man meistens PEG (Parsing Expression Grammar).
Für die Binär Operationen sähe der PEG Code so aus:
```peg
expr:
	  expr '+' term
	| expr '-' term
	| term  
term:
	  term '*' atom
	| term '/' atom
	| atom  
atom:
	  IDENT
	| NUMBER
	| '(' expr ')'
```
Es gibt sozusagen Layer die einen Name haben und in diesen beschreibt man die Muster die in dem Layer auftreten dürfen.
### PEG  Code für meine Sprache 
Hier ist die ganze Grammatik für meine Sprache:
```peg
chunk:
	  [(statement)*]
statement:
	  'let' ident '=' expr
	| 'def' ident '(' [ident (',' ident)*] ')' block
	| if-statement
	| 'while' expr block
	| 'return' expr
	| ident '(' [expr (',' expr)*] ')'
	| ident '=' expr
	| block
if-statement:
	  'if' expr block
	| 'if' expr block 'else' block
	| 'if' expr block 'else' if-statement
block:
	  '{' [(statement)*] '}'
expr:
	| expr '(' [expr (',' expr)*] ')'
	| comp  
comp:
	  comp '==' arith
	| comp '!=' arith
	| comp '<' arith
	| comp '>' arith
	| comp '<=' arith
	| comp '>=' arith
	| arith
arith:
	  arith '+' term
	| arith '-' term
	| arith
term:
	  term '*' exp
	| term '/' exp
	| term '%' exp
	| exp
exp:
	  exp '^' not
	| not
not:
	  '!' not
	| neg
neg:
	  '-' neg
	| atom
atom:
	  IDENT
	| NUMBER
	| STRING
	| '(' expr ')'
```
### In Rust
Der Rust Code ist fast so wie der PEG Code, nur dass ich die Binären Operationen vereinfacht habe, was den ganzen Code kleiner macht und mir erlaubt später vielleicht mehr Operatoren hinzuzufügen.
```rust
struct Chunk(Vec<Located<Statement>>);
struct Block(Vec<Located<Statement>>);
enum Statement {
	Block(Block),
	Let {
		ident: Located<String>,
		expr: Located<Expression>,
	},
	Assign {
		ident: Located<String>,
		expr: Located<Expression>,
	},
	Call {
		ident: Located<String>,
		args: Vec<Located<Expression>>,
	},
	  
	Def {
		ident: Located<String>,
		params: Vec<Located<String>>,
		body: Located<Block>,
	},
	If {
		cond: Located<Expression>,
		case: Located<Block>,
		else_case: Option<Located<Block>>,
	},
	While {
		cond: Located<Expression>,
		body: Located<Block>,
	},
	  
	Return(Located<Expression>),
}
enum Expression {
	Atom(Atom),
	Binary {
		op: BinaryOperator,
		left: Box<Located<Self>>,
		right: Box<Located<Self>>,
	},
	Unary {
		op: UnaryOperator,
		right: Box<Located<Self>>,
	},
	Call {
		head: Box<Located<Self>>,
		args: Vec<Located<Self>>,
	},
}
enum BinaryOperator {
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
}
enum UnaryOperator {
	Minus, // -
	Exclamation, // !
}
enum Atom {
	Ident(String),
	Number(f64),
	String(String),
	Expression(Box<Located<Expression>>),
}
```
Dafür habe ich auch folgenden Code angelegt um die Operatoren Reihenfolge einfach darzustellen können und verändern zu können.
```rust
impl BinaryOperator {
	const LAYER: &'static [&'static [Self]] = &[
		&[Self::Ampersand, Self::Pipe],
		&[
			Self::EqualEqual,
			Self::ExclamationEqual,
			Self::Less,
			Self::Greater,
			Self::LessEqual,
			Self::GreaterEqual,
		],
		&[Self::Plus, Self::Minus],
		&[Self::Star, Self::Slash, Self::Percent],
		&[Self::Exponent],
	];
	fn layer(layer: usize) -> Option<&'static [Self]> {
		Self::LAYER.get(layer).copied()
	}
	fn token(token: &Token) -> Option<Self> {
		match token {
			Token::Plus => Some(Self::Plus),
			Token::Minus => Some(Self::Minus),
			Token::Star => Some(Self::Star),
			Token::Slash => Some(Self::Slash),
			Token::Percent => Some(Self::Percent),
			Token::Exponent => Some(Self::Exponent),
			Token::EqualEqual => Some(Self::EqualEqual),
			Token::ExclamationEqual => Some(Self::ExclamationEqual),
			Token::Less => Some(Self::Less),
			Token::Greater => Some(Self::Greater),
			Token::LessEqual => Some(Self::LessEqual),
			Token::GreaterEqual => Some(Self::GreaterEqual),
			Token::Ampersand => Some(Self::Ampersand),
			Token::Pipe => Some(Self::Pipe),
			_ => None,
		}
	}
}
impl UnaryOperator {
	pub const LAYER: &'static [&'static [Self]] = &[
		&[Self::Exclamation], 
		&[Self::Minus]
	];
	pub fn layer(layer: usize) -> Option<&'static [Self]> {
		Self::LAYER.get(layer).copied()
	}
	pub fn token(token: &Token) -> Option<Self> {
		match token {
			Token::Minus => Some(Self::Minus),
			Token::Exclamation => Some(Self::Exclamation),
			_ => None,
		}
	}
}
```
Die Konstante `LAYER` in beiden Implementierungen hat ein statische Referenz zu einem Array wo Arrays mit Operatoren drin sind (auch statische Referenz). Dieser ist dafür da später beim parsen die Layer nach Index identifizieren zu können. Wenn der Parser bei Layer `n` ist, weiß er, dass er als nächstes zu Layer `n + 1` muss. Diese Optimierung habe nach öfteren Parser kreieren selber entwickelt, dennoch kann ich mir vorstellen, dass das schon jemanden vor mir erfunden hat, aber so lange ich das nicht weiß, werde ich mir immer wieder selber auf die Schulter klopfen :)
### Errors
Für die Fehler die beim Parsen enstehen können habe ich folgenden Error `enum` geschrieben:
```rust
enum ParseError {
	UnexpectedEOF, // unexpected end of file
	UnexpectedToken(Token), // unexpected token $0
	ExpectedToken { // expected $expected, got $got
		expected: Token,
		got: Token
	},
}
```
Das sind alle Fehler-Varianten, die bei diesem Parser auftreten können, doch andere Parser können natürlich mehr besitzen, was meistens der Fall für kompliziertere Sprachen ist, aber da ich nur eine Beispielsprache mache, ist das genug.

Weiter zum [Compiler](Compiler.md)