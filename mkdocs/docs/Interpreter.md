Der Interpreter muss nun den Byte Code des Compilers interpretieren. Teil des Interpreters habe ich schon im [Compiler Abschnitt](Compiler.md) erklärt, da der Compiler abhängig von dem Interpreter ist und anders herum auch. Beide Implementationen müssen auf einander abgestimmt sein, sonst kommt es zu einem Fehlerhaften ausführen und unerwarteten Problemen.
## Stack
Ich benutze das Wort Stack im folgendem sehr oft deswegen gibt es eine kurze ChatGPT Zusammenfassung davon:

> In der Informatik bezeichnet ein "Stack" eine abstrakte Datenstruktur, die nach dem LIFO (Last In, First Out) Prinzip funktioniert. Das bedeutet, dass das zuletzt hinzugefügte Element auch als erstes entfernt wird. Ein Stack kann als Stapel von Objekten betrachtet werden, ~~*bei dem nur das oberste Element zugänglich ist*~~.
> 
> Ein Stack unterstützt im Allgemeinen zwei grundlegende Operationen:
> - Push: Dies bedeutet, ein Element oben auf den Stapel zu legen. Es wird häufig auch als "Hinzufügen" oder "Einlegen" bezeichnet.
> - Pop: Dies bedeutet, das oberste Element vom Stapel zu entfernen. Es wird häufig auch als "Entfernen" oder "Herausnehmen" bezeichnet.
> 
> Neben diesen grundlegenden Operationen unterstützt ein Stack oft auch eine dritte Operation:
> - Peek (auch Top genannt): Diese Operation ermöglicht es, das oberste Element des Stapels zu betrachten, ohne es zu entfernen.
> 
> Stacks werden häufig in der Informatik verwendet, insbesondere in der Programmierung, um die Reihenfolge von Operationen zu verfolgen, wie zum Beispiel bei der Auswertung von Ausdrücken in umgekehrter polnischer Notation (Reverse Polish Notation, RPN), der Verwaltung von Funktionsaufrufen oder dem Durchlaufen von Datenstrukturen wie Bäumen (zum Beispiel beim Tiefensuchalgorithmus).
_von: https://chat.openai.com_

Diese Erklärung ist sehr akkurat, außer dass in meinem Fall jedes Element im Stack angeschaut werden kann, und nicht nur das oberste.
## Value
Um alle Werte darstellen zu können für den Interpreter habe ich folgendes `enum` geschrieben:
```rust
enum Value {
    #[default]
    Null,
    Number(f64),
    Boolean(bool),
    String(Rc<str>),
    Function(Rc<Function>),
}
enum Function {
    NativeFunction(NativeFunction),
    Function(Rc<Closure>),
}
type NativeFunction = fn(&mut Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error>>;
```
- `Null`: ist ein leerer Wert, andere Sprachen nennen ihn None oder Nil.
- `Number`: eine Dezimalzahl
- `Boolean`: ein Bit der Wahr oder Falsch darstellt
- `String`: eine Referenz zu einem String
- `Function`: Eine Referenz zu einer Funktion
	- `NativeFunction`: Eine in Rust geschriebene Funktion
	- `Function`: Eine Referenz zu einem Closure
## Interpreter
In meinem Interpreter benutze ich zwei Stacks: Einmal den Call Stack, und für jeden Call Stack noch einen Register Stack. Der Call Stack ist dafür da sich die Funktionsaufrufe zu merken die während des Ausführen eine Codes zu merken. Der Register Stack ist dafür da sich die Werte von Variablen oder die Werte von Operationen zu speichern, welche relevant sind.
```rust
struct Interpreter {
    call_stack: Vec<CallFrame>,
    globals: HashMap<String, Value>,
}
struct CallFrame {
    closure: Rc<Closure>,
    ip: Address,
    stack: Vec<Rc<RefCell<Value>>>,
    dst: Option<Register>,
}
```
Der Interpreter hat noch zusätzlich eine Globale Umgebung (also eine Tabelle mit Identifier-Value Paaren). Ein Call Frame muss mit einer Referenz zu einem Closure natürlich auch einen Instruction Pointer haben, der zeigt wo im Code der Frame sich gerade befindet, den oben genannten Register Stack, wo ein Register eigentlich nur eine Zellen-Referenz zu einem Value ist, und noch ein optionales Result Register (das Zielregister aus dem vorherigen Call Frame).
### Errors
Natürlich können auch Fehler während der Ausführung des Programs auftreten, weswegen man einen Error Typen benötigt.
```rust
enum RunTimeError {
    Binary {
        left: &'static str,
        right: &'static str,
    },
    Unary {
        right: &'static str,
    },
    CannotCall(&'static str),
    Custome(String),
}
```
`&'static str` heißt so viel wie ein String der für das ganze Program statisch, also gleich, bleibt. In diesem Fall sind das Strings für die Namen der Typen, was in diesem Fall nur folgende Typen sein kann:
- `null`
- `number`
- `boolean`
- `string`
- `function`
Für ein gute Programmiersprache fehlt definitiv noch eine Art von Kollektionstypen, wie eine Liste und/oder eine Art Objekt, auch Dictionary in Python oder Table in Lua genannt. Aber das hier ist nur eine Beispiel Sprache, also habe ich diese jetzt erst mal raus gelassen. (Aber in [Luna](https://github.com/sty00a4-code/luna) habe ich tatsächlich beides implementiert)
### Implementierung
#### Funktions Aufrufung
Als erstes ist es wichtig das der Interpreter Funktionen aufrufen kann. Das heißt er braucht eine Referenz zu einem Closure, die Argumente, und ein mögliches Result Register.
```rust
fn call_closure(&mut self, closure: &Rc<Closure>, args: Vec<Value>, dst: Option<Register>) {
	let mut stack = Vec::with_capacity(closure.registers as usize + 1);
	let args = &args[0..stack.capacity().min(args.len())];
	let args_len = args.len();
	stack.extend(args.iter().map(|v| Rc::new(RefCell::new(v.clone()))));
	stack.extend(
		(args_len..=closure.registers as usize).map(|_| Rc::new(RefCell::new(Value::default()))),
	);
	self.call_stack.push(CallFrame {
		closure: Rc::clone(closure),
		ip: 0,
		stack,
		dst,
	});
}
```
Hier lege ich ein neues Call Frame an und füge es dem Call Stack hinzu. Der Stack wird mit einer fixen Kapazität angelegt (Performance Gründe), dann werden die Argumente zu der Stack Kapazität eingeschränkt. Danach wird der Stack mit den Argumenten gefüllt und falls noch etwas Platz übrig ist müssen dort Null Values drinnen sein.
#### Funktions Wiedergabe
Um aus einer Funktion rauszukommen habe ich folgende Funktion geschrieben:
```rust
pub fn return_call(&mut self, src: Option<Register>) -> Option<Value> {
	let top_frame = self.call_stack.pop().expect("no frame on stack");
	if let Some(prev_frame) = self.call_frame_mut() {
		if let Some(dst) = top_frame.dst {
			let value = if let Some(src) = src {
				top_frame
					.register(src)
					.expect("source not found")
					.borrow()
					.clone()
			} else {
				Value::default()
			};
			let dst = prev_frame.register(dst).expect("location not found");
			*dst.borrow_mut() = value;
		}
	}
	if let Some(src) = src {
		return Some(
			top_frame
				.register(src)
				.expect("source not found")
				.borrow()
				.clone(),
		);
	}
	None
}
```
Hier wird das oberste Call Frame weggenommen und geschaut ob es einen Result Register hat. Wenn das der Fall ist wird der Wert in dem `src` Register oder Null in das Result Register getan. Der gleiche Wert wird auch aus der Funktion wiedergegeben.
#### Anweisungen
Um in dem Program fortzuschreiten benötigen wir eine Funktion, die die jetzige Anweisung ausführt und dann zur nächsten geht.
```rust
fn run(&mut self, closure: &Rc<Closure>) -> Result<Option<Value>, Located<RunTimeError>> {
	let offset = self.call_stack.len();
	self.call_closure(closure, vec![], None);
	loop {
		let value = self.step()?;
		if self.call_stack.len() <= offset || self.call_stack.is_empty() {
			return Ok(value);
		}
	}
}
```
Die `run` Funktion hier ruft die `step` Funktion immer wieder auf bis der Call Stack wieder die gleich Größe hat wie vor dem Aufrufen oder 0 hat, sprich er ist leer. Da ein Wert wiedergegeben werden kann wird der letzte Wert der `step` Funktion dafür genommen
```rust
fn step(&mut self) -> Result<Option<Value>, Located<RunTimeError>> {
	let Located {
		value: bytecode,
		pos,
	} = self
		.call_frame()
		.expect("no call frame on stack")
		.instr()
		.expect("ip out of range")
		.clone();
	self.call_frame_mut().expect("no call frame on stack").ip += 1; // advance the instruction pointer
	match bytecode {
		...
	}
	Ok(None)
}
```
Hier wird die momentane Anweisung genommen und der Instruction Pointer um eins erhöht. Die genommene Anweisung muss dann nur noch bearbeitet werden in einem `match` Zweig.
##### None
```rust
ByteCode::None => {}
```
Macht nichts...
##### Jump
```rust
ByteCode::Jump { addr } => {
	self.call_frame_mut().expect("no call frame on stack").ip = addr;
}
```
Setzt den Instruction Pointer auf die Adresse
##### Jump If
```rust
ByteCode::JumpIf {
	not: false,
	cond,
	addr,
} => {
	let cond = self
		.register(cond)
		.expect("register not found")
		.borrow()
		.clone();
	if bool::from(&cond) {
		self.call_frame_mut().expect("no call frame on stack").ip = addr;
	}
}
ByteCode::JumpIf {
	not: true,
	cond,
	addr,
} => {
	let cond = self
		.register(cond)
		.expect("register not found")
		.borrow()
		.clone();
	if !bool::from(&cond) {
		self.call_frame_mut().expect("no call frame on stack").ip = addr;
	}
}
```
Nimmt den Wert in dem `cond` Register, wandelt ihn in ein Boolean um und setzt den Instruction Pointer auf die Adresse. Das `not` verneint den booleschen Wert natürlich.
##### Call
```rust
ByteCode::Call {
	func,
	offset,
	args_len,
	dst,
} => {
	let func = self
		.register(func)
		.expect("register not found")
		.borrow()
		.clone();
	let mut args: Vec<Value> = Vec::with_capacity(args_len as usize);
	args.extend((offset..offset + args_len as Register).map(|reg| {
		self.register(reg)
			.expect("register not found")
			.borrow()
			.clone()
	}));
	match func {
		Value::Function(function) => {
			self.call(&function, args, &pos, dst)
				.map_err(|err| err.map(|err| RunTimeError::Custome(err.to_string())))?;
		}
		value => return Err(Located::new(RunTimeError::CannotCall(value.typ()), pos)),
	}
}
```
Nimmt den Wert in dem `func` Register, sammelt die Argumente aus den Registern und ruft die Funktion auf wenn von Typ `function` ist, sonst gibt er einen `RunTimeError` wieder.
##### Return
```rust
ByteCode::Return { src } => return Ok(self.return_call(src)),
```
Gibt den Wert aus dem `src` Register wieder wenn es gegeben ist.
##### Move
```rust
ByteCode::Move { dst, src } => {
	let src = self
		.register(src)
		.expect("register not found")
		.borrow()
		.clone();
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = src;
}
```
Kopiert den Wert aus dem `src` Register und setzt den Wert des `dst` Register darauf.
##### String und Number
```rust
ByteCode::String { dst, addr } => {
	let string = self
		.call_frame()
		.expect("no call frame on stack")
		.closure
		.string(addr)
		.expect("string not found")
		.clone();
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = Value::String(string.into());
}
ByteCode::Number { dst, addr } => {
	let number = self
		.call_frame()
		.expect("no call frame on stack")
		.closure
		.number(addr)
		.expect("number not found")
		.clone();
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = Value::Number(number);
}
```
Kopiert den konstanten Wert aus dem jeweiligen Konstanten Pool und setzt das `dst` Register auf diesen Wert (umgewandelt in ein Value natürlich)
##### Closure
```rust
ByteCode::Closure { dst, addr } => {
	let closure = Rc::clone(
		self.call_frame()
			.expect("no call frame on stack")
			.closure
			.closure(addr)
			.expect("closure not found"),
	);
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = Value::Function(Rc::new(Function::Function(closure)));
}
```
Erstellt eine neue Referenz zu dem Closure aus dem Closure Konstanten Pool und setzt das `dst` Register auf diesen Wert in Form einer Funktion
##### Global
```rust
ByteCode::Global { dst, addr } => {
	let value = {
		let string = self
			.call_frame()
			.expect("no call frame on stack")
			.closure
			.string(addr)
			.expect("string not found");
		self.globals.get(string).cloned().unwrap_or_default()
	};
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = value;
}
```
Kopiert den Wert aus der Globalen Umgebung von dem String des String Konstanten Pools und setzt das `dst` Register auf diesen Wert.
##### Set Global
```rust
ByteCode::SetGlobal { addr, src } => {
	let value = self.register(src).expect("register not found").borrow().clone();
	let ident = self
		.call_frame()
		.expect("no call frame on stack")
		.closure
		.string(addr)
		.expect("string not found")
		.clone();
	let old_value = {
		self.globals.get_mut(&ident)
	};
	if let Some(old_value) = old_value {
		*old_value = value;
	} else {
		self.globals.insert(ident, value);
	}
}
```
Nimmt den Wert aus der Globalen Umgebung von dem String des String Konstanten Pools und setzt diesen Wert auf den Wert aus dem `src` Register. Falls der Eintrag in der Globalen Umgebung nicht existiert, wird ein neuer Eintrag erstellt mit dem Wert aus dem `src` Register.
##### Binary
```rust
ByteCode::Binary {
	op,
	dst,
	left,
	right,
} => {
	let left = self
		.register(left)
		.expect("register not found")
		.borrow()
		.clone();
	let right = self
		.register(right)
		.expect("register not found")
		.borrow()
		.clone();
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = match op {
		...
	};
}
```
Nimmt die Werte aus den `left` Register und `right` Register, führt die Binary Operation aus, und setzt den Wert in dem `dst` Register auf das Ergebnis.
##### Binary
```rust
ByteCode::Unary { op, dst, src } => {
	let right = self
		.register(src)
		.expect("register not found")
		.borrow()
		.clone();
	let mut dst = self.register(dst).expect("register not found").borrow_mut();
	*dst = match op {
		...
	};
}
```
Nimmt den Wert aus dem `right` Register, führt die Unary Operation aus, und setzt den Wert in dem `dst` Register auf das Ergebnis.
### Globale Funktionen
Die Programmiersprache funktioniert schon ganz gut, dennoch kann man nicht viel damit interagieren. Zum Beispiel fehlen Funktionen wie `print`. Deswegen müssen vor dem Start des Interpretierens globale Funktionen erstellt werden.
```rust
fn run(closure: &Rc<Closure>) -> Result<Option<Value>, Located<RunTimeError>> {
    let mut interpreter = Interpreter::default();
    std_globals(&mut interpreter.globals);
    interpreter.run(closure)
}

fn std_globals(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "print".into(),
        Value::Function(Rc::new(Function::NativeFunction(_print))),
    );
}

fn _print(_: &mut Interpreter, args: Vec<Value>) -> Result<Value, Box<dyn Error>> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    Ok(Value::default())
}
```
Die `run` Funktion nimmt ein Closure, erstellt einen Interpreter, lässt `std_globals` standard globale Variablen erstellen, und führt den Code aus. Hier habe ich die `print` Funktion definiert dammit man endlich ein `Hello World!`-Program schreiben kann.
```
print("Hello World!")
```