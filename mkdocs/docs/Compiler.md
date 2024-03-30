Wie ich schon in dem Abschnitt "[Was ist ein Compiler](docs/Was%20ist%20ein%20Compiler)" erklärt habe, muss der AST jetzt nur noch in einen Byte Code umgewandelt werden.
## Closure
Ein Closure ist ein eigene Code Instanz die den Byte Code speichert, sowie die Anzahl an benötigten Registern und Konstanten. Die folgende Struktur ist ein Closure in meinem Compiler
```rust
struct Closure {
	code: Vec<Located<ByteCode>>,
	registers: u16,
	strings: Vec<String>,
	numbers: Vec<f64>,
	closures: Vec<Rc<Self>>,
}
```
Hier wird ein Array von Byte Codes gespeichert, die Register Anzahl und Konstanten, also Zahlen, Strings, und Referenzen zu anderen Closures die in diesem Teil des Source Codes entstehen können. Closures werden vom Interpreter Aufgerufen in einem Call Stack
## Byte Code
### Im Interpreter
Da Compiler und Interpreter funktional sehr abhängig von einander sind, muss ich vorher ein paar Sachen klar stellen, wie der Interpreter den Byte Code tatsächlich ausführt.
Der Interpreter den ich schreiben werde, hat einen sogenannten Call Stack, der dafür zuständig ist sich zu merken, welche Closure gerade ausgeführt wird. In dem Call Stack werden Call Frames gespeichert, die wie folgt strukturiert sind:
```rust
struct CallFrame {
	closure: Rc<Closure>,
	ip: Address,
	stack: Vec<Rc<RefCell<Value>>>,
	dst: Option<Register>,
}
```
Dieser hat eine Referenz zu dem Closure der gerade Ausgeführt wird, einen Instruction Pointer der sagt wo im Code der Interpreter sich gerade befindet, einen Stack auf dem Werte gespeichert werden, um mit diesen Operationen auszuführen, und einen Wiedergabe Register des Stacks des Call Frames vor diesem Call Frame.
### Register
In meinem Compiler benutze ich so genannte Register um Werte zu speichern und daran Operationen durchzuführen später beim Interpreter. Um diese Register anzusprechen muss man sie natürlich identifizieren können mit Hilfe eines Indexes. Dieser Index kann von 0 bis hoch zu einer beliebigen Größe gehen. 
```rust
type Register = u16;
```
Hier definiere ich Typen Aliase, was bedeutet, wenn ich jetzt `Register` schreibe, ersetzt Rust das mit `u16` (16 Bit unsigned integer).
### Adressen
Was auch später wichtig für den Interpreter wird ist, wie er zu einer beliebigen Anweisung springen kann.  
```rust
type Address = u32;
```
In meinem Interpreter werden die Positionen der Anweisungen mit einem `u32` (32 Bit unsigned integer) angesprochen.
### Anweisungen
Jetzt komme ich zu den tatsächlichen Byte Code den ich für diese Sprache geschrieben habe:
```rust
enum ByteCode {
    #[default]
    None,
    
    Jump {
        addr: Address,
    },
    JumpIf {
        not: bool,
        cond: Register,
        addr: Address,
    },
    
    Call {
        func: Register,
        offset: Register,
        args_len: u8,
        dst: Option<Register>,
    },
    Return {
        src: Option<Register>,
    },
    
    Move {
        dst: Register,
        src: Register,
    },
    String {
        dst: Register,
        addr: Address,
    },
    Number {
        dst: Register,
        addr: Address,
    },
    Closure {
        dst: Register,
        addr: Address,
    },
    Global {
        dst: Register,
        addr: Address,
    },
    SetGlobal {
        addr: Address,
        src: Register
    },
    
    Binary {
        op: BinaryOperation,
        dst: Register,
        left: Register,
        right: Register,
    },
    Unary {
        op: UnaryOperation,
        dst: Register,
        src: Register,
    },
}
enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
    And,
    Or,
}
enum UnaryOperation {
    Neg,
    Not,
}
```
Das sind alle nötigen Anweisungen für meine Sprache.
#### Jump
```rust
Jump {
    addr: Address,
}
```
Sagt dem Interpreter, dass er zu einer bestimmten Adresse springen soll.
#### Jump If
```rust
JumpIf {
	not: bool,
	cond: Register,
    addr: Address,
}
```
Sagt dem Interpreter, dass er zu einer bestimmten Adresse springen soll, falls der Wert in dem Register `cond` wahr ist. Dieser Boolesche Wert kann mit dem `not = true` verneint werden.
#### Call
```rust
Call {
	func: Register,
	offset: Register,
	args_len: u8,
	dst: Option<Register>,
},
```
Ruft die Funktion in Register `func` auf (wenn der Wert überhaupt eine Funktion ist) und gibt optional den Wiedergabewert der Funktion in ein bestimmtes Register zurück. Die Argumente für diesen Call werden mit `offset`, wo die Argumente im Stack anfangen, und `args_len`, wie viele Argumente eingenommen werde, angegeben.
#### Return
```rust
Return {
	src: Option<Register>,
},
```
Geht aus dem jetzigen Call raus mit einen Optionalen Wiedergabewert.
#### Move
```rust
Move {
	dst: Register,
	src: Register,
},
```
Kopiert den Wert von Register `src` zu `dst`.
#### String, Number und Closure
```rust
String {
	dst: Register,
	addr: Address, // address to string pool
},
Number {
	dst: Register,
	addr: Address, // address to number pool
},
Closure {
	dst: Register,
	addr: Address, // address to closure reference pool
},
```
Alle drei Anweisungen sagen dem Interpreter er soll eine Konstante aus dem Konstanten Pool in das `dst` Register kopieren. Diese Konstanten sind jeweils Zahlen (`f64`), Strings und Closures. Diese Werden jeweils in jedem Closure gespeichert.
#### Global
```rust
Global {
	dst: Register,
	addr: Address,
},
```
Diese Anweisung funktioniert fast wie die für die Konstanten, dennoch wird hier nicht aus dem Konstanten Pool genommen, sondern aus der Globalen Umgebung. Dafür wird ein String aus den Strings des Closures kopiert und dann wird dieser String als Identifier in der Globalen Umgebung gesucht. Wenn kein passender Schlüssel gefunden wird, wird `null` in das Register stattdessen geladen.
#### Set Global
```rust
SetGlobal {
	addr: Address,
	src: Register
},
```
Setzt den Wert des Schlüssels (aus dem String Pool) der Globalen Umgebung auf den Wert in dem `src` Registers.
#### Binary und Unary
```rust
Binary {
	op: BinaryOperation,
	dst: Register,
	left: Register,
	right: Register,
},
Unary {
	op: UnaryOperation,
	dst: Register,
	src: Register,
},
```
Diese Anweisungen führen jeweils ihre Operationen auf den Registern `left` und `right`, bzw. für Unary nur `right`, und setzt den resultierenden Wert in das Register `dst`. 
Hier nochmal die Operationen:
```rust
enum BinaryOperation {
    Add, // left + right
    Sub, // left - right
    Mul, // left * right
    Div, // left / right
    Mod, // left % right
    Pow, // left ^ right
    EQ, // left == right
    NE, // left != right
    LT, // left < right
    GT, // left > right
    LE, // left <= right
    GE, // left >= right
    And, // left & right
    Or, // left | right
}
enum UnaryOperation {
    Neg, // -right
    Not, // !right
}
```
## Compiler
Jetzt kommen wir zum tatsächlichen Compilierungsvorgang. Der Compiler den ich schreibe basiert auf einem Frame und Scope Prinzip.
```rust
struct Compiler {
    frames: Vec<Frame>, // frame stack
}
struct Frame {
    closure: Closure, // editable closure
    registers: Register, // currently used registers
    scopes: Vec<Scope>, // scope stack
}
struct Scope {
    locals: HashMap<String, Register>, // table of variable register pairs
    offset: Register, // register stack offset
}
```
Der Compiler hat einen Frame Stack und compiliert immer nur den letzen Frame. Jeder Frame hat einen veränderbaren Closure, die Anzahl an momentan benutzen Registern und ein Scope Stack. Jeder Scope hat eine Tabelle wo Variablen und ihre Register Indexes gespeichert werden, sowie den Register Stack Offset, der angibt an Welchem Register Index der Scope anfängt.
### Implementierung
```rust
trait Compilable {
    type Output;
    type Error;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error>;
}
```
Hier definiere ich einen Trait der den Implementierer zum Compilieren wichtige Informationen sowie Funktionen implementieren lässt.
#### Chunk
```rust
impl Compilable for Located<Chunk> {
    type Output = Closure;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        compiler.push_frame(); // new frame
        for stat in self.value.0 {
            stat.compile(compiler)?; // compile every statement
        }
        compiler.frame_mut().closure.write(ByteCode::Return { src: None }, self.pos); // mendatory null return
        let frame = compiler.pop_frame().unwrap(); // pop frame
        Ok(frame.closure)
    }
}
```
Das hier ist die Implementierung für den Chunk AST. Dieser gibt ein Closure wieder oder als Error einen Compile Error, der momentan noch leer ist.
Der Code in Schritten erklärt:
1. Werfe ein neuen Frame auf den Frame Stack
2. Iteriere über jedes Statement und compiliere es
3. Schreibe eine Return Anweisung am Ende in das Closure
4. Nimm den Frame den du beim 1. Schritt erstellt hast vom Stack
5. Gib den Closure des Frames wieder
#### Block
Für den Block AST ist die Implementierung ähnlich zu der des Chunk AST, nur dass kein neues Frame auf den Frame Stack geworfen wird sonder ein neuer Scope auf den Scope Stack.
#### Statement
```rust
impl Compilable for Located<Statement> {
    type Output = Option<Register>;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        let Located { value: stat, pos } = self;
        match stat {
            ...
        }
    }
}
```
Beim Statement habe ich mehrere Varianten die ich alle verschieden compilieren muss. Der Output ist hier ein Optionales Register, denn das Statement kann ein Return Statement sein welche ein den Wert eines Registers wiedergeben kann.
##### Block
```rust
Statement::Block(block) => Located::new(block, pos).compile(compiler)
```
Compiliere einfach den Block
##### Let
```rust
Statement::Let {
	ident:
		Located {
			value: ident,
			pos: _,
		},
	expr,
} => {
	let reg = compiler.frame_mut().new_local(ident);
	let src = expr.compile(compiler)?;
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::Move { dst: reg, src }, pos);
	Ok(None)
}
```
1. Erstelle einen neue lokale Variable
2. Compiliere die Expression
3. Schreibe in den jetzigen Closure eine Move Anweisung von dem Result Register  der Expression aus dem 2. Schritt zum Register der Variable aus dem 1. Schritt
##### Assign
```rust
Statement::Assign {
	ident:
		Located {
			value: ident,
			pos: ident_pos,
		},
	expr,
} => {
	let reg = compiler.frame_mut().local(&ident);
	let src = expr.compile(compiler)?;
	if let Some(reg) = reg {
		compiler
			.frame_mut()
			.closure
			.write(ByteCode::Move { dst: reg, src }, pos);
	} else {
		let addr = compiler.frame_mut().closure.new_string(ident);
		compiler
			.frame_mut()
			.closure
			.write(ByteCode::SetGlobal { addr, src }, pos);
	}
	Ok(None)
}
```
1. Finde das Register der lokalen Variable (falls es diese gibt)
2. Compiliere die Expression
3. Wenn du das Register aus dem 1. Schritt gefunden hast
	1. Schreibe in den jetzigen Closure eine Move Anweisung von dem Result Register  der Expression aus dem 2. Schritt zum Register der Variable aus dem 1. Schritt
4. Falls du das Register aus dem 1. Schritt nicht gefunden hast
	1. Lege eine neue String Konstante diese Identifiers an
	2. Schreibe in den jetzigen Closure eine Set Global Anweisung mit dem Result Register der Expression aus dem 2. Schritt und der Adresse der String Konstante aus dem 4.1. Schritt
##### Call
```rust
Statement::Call {
	ident:
		Located {
			value: ident,
			pos: ident_pos,
		},
	args,
} => {
	let func = if let Some(reg) = compiler.frame_mut().local(&ident) {
		reg
	} else {
		let addr = compiler.frame_mut().closure.new_string(ident);
		let dst = compiler.frame_mut().new_register();
		compiler
			.frame_mut()
			.closure
			.write(ByteCode::Global { dst, addr }, ident_pos);
		dst
	};
	let args_len = args.len() as u8;
	let offset = compiler.frame().registers;
	compiler.frame_mut().registers += args_len as Register;
	for (reg, arg) in (offset..offset + args_len as Register).zip(args) {
		let pos = arg.pos.clone();
		let src = arg.compile(compiler)?;
		compiler
			.frame_mut()
			.closure
			.write(ByteCode::Move { dst: reg, src }, pos);
	}
	compiler.frame_mut().closure.write(
		ByteCode::Call {
			func,
			offset,
			args_len,
			dst: None,
		},
		pos,
	);
	Ok(None)
}
```
1. Finde das Register der Funktion
2. Merke dir das Offset als die jetzige Register Anzahl
3. Addiere die Anzahl an Argumenten auf die Register Anzahl drauf
4. Iteriere über jedes Argument
	1. Compiliere das Argument und speicher das Result Register
	2. Schreibe in den jetzigen Closure eine Move Anweisung von dem Result Register aus dem 4.1 Schritt zu dem entsprechenden Argument Register
5. Schreibe in den jetzigen Closure eine Call Anweisung mit dem Register der Funktion aus dem 1. Schritt, das Offset aus dem 2. Schritt, die Argumenten Länge, und mit einem leeren Result Register.
##### Def
```rust
Statement::Def {
	ident:
		Located {
			value: ident,
			pos: _,
		},
	params,
	body,
} => {
	let reg = compiler.frame_mut().new_local(ident);
	let addr = {
		compiler.push_frame();
		for Located {
			value: param,
			pos: _,
		} in params
		{
			compiler.frame_mut().new_local(param);
		}
		body.compile(compiler)?;
		compiler
			.frame_mut()
			.closure
			.write(ByteCode::Return { src: None }, pos.clone());
		let frame = compiler.pop_frame().unwrap();
		let closure = Rc::new(frame.closure);
		compiler.frame_mut().closure.new_closure(closure)
	};
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::Closure { dst: reg, addr }, pos);
	Ok(None)
}
```
1. Erstelle einen neue lokale Variable in den jetzigen Closure
2. Compiliere die die Parameter und den Body in einem neuen Frame
	1. Füge einen neuen Frame auf den Frame Stack hinzu
	2. Iteriere über die Parameter
		1. Erstelle eine neue lokale Variable in den neuen Closure
	3. Compiliere den Body
	4. Schreibe ein leere Return Anweisung in den neuen Closure 
	5. Nehme den neuen Frame wieder vom Frame Stack runter und speicher eine Referenz zu dem Closure
	6. Lege eine neue Closure Konstante in dem Closure Konstanten Pool an
3. Schreibe in den jetzigen Closure eine Closure Anweisung mit der Adresse aus dem 2. Schritt in das Register der Variable aus dem 1. Schritt
##### If
```rust
Statement::If {
	cond,
	case,
	else_case,
} => {
	let cond = cond.compile(compiler)?;
	let check_addr = compiler
		.frame_mut()
		.closure
		.write(ByteCode::default(), pos.clone());
	case.compile(compiler)?;
	let case_exit_addr = compiler.frame_mut().closure.write(ByteCode::default(), pos);
	let else_addr = compiler.frame_mut().closure.code.len() as Address;
	if let Some(else_case) = else_case {
		else_case.compile(compiler)?;
	}
	let exit_addr = compiler.frame_mut().closure.code.len() as Address;
	compiler.frame_mut().closure.overwrite(
		check_addr,
		ByteCode::JumpIf {
			not: true,
			cond,
			addr: else_addr,
		},
	);
	compiler
		.frame_mut()
		.closure
		.overwrite(case_exit_addr, ByteCode::Jump { addr: exit_addr });
	Ok(None)
}
```
1. Compiliere die `cond` Expression und merke dir das Result Register
2. Schreibe eine leere Anweisung in den jetzigen Closure und merke dir dessen Adresse
3. Compiliere den `case` Block
4. Schreibe eine leere Anweisung in den jetzigen Closure und merke dir dessen Adresse
5. Merke dir die Adresse der nächsten Anweisung
6. Falls es einen `else_case` gibt
	1. Compiliere den `else_case` Block
7. Merke dir die Adresse der Anweisung nach der Letzten
8. Überschreibe die Anweisung bei der Adresse aus dem 2. Schritt mit einer Jump If Anweisung wo `not` wahr ist, das Konditionsregister das Result Register aus dem 1. Schritt ist, und die Jump Adresse die Adresse aus dem 5. Schritt ist
9. Überschreibe die Anweisung bei der Adresse aus dem 4. Schritt mit einer Jump Anweisung wo die Jump Adresse die Adresse aus dem 7. Schritt ist
##### While
```rust
Statement::While { cond, body } => {
	let cond_addr = compiler.frame_mut().closure.code.len() as Address;
	let cond = cond.compile(compiler)?;
	let check_addr = compiler
		.frame_mut()
		.closure
		.write(ByteCode::default(), pos.clone());
	body.compile(compiler)?;
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::Jump { addr: cond_addr }, pos);
	let exit_addr = compiler.frame_mut().closure.code.len() as Address;
	compiler.frame_mut().closure.overwrite(
		check_addr,
		ByteCode::JumpIf {
			not: true,
			cond,
			addr: exit_addr,
		},
	);
	Ok(None)
}
```
1. Merke dir die Adresse der nächsten Anweisung
2. Compiliere die `cond` Expression und merke dir das Result Register
3. Schreibe eine leere Anweisung in den jetzigen Closure und merke dir dessen Adresse
4. Compiliere den `body` Block
5. Schreibe eine Jump Anweisung in den jetzigen Closure mit der Adresse aus dem 1. Schritt
7. Merke dir die Adresse der nächsten Anweisung
8. Überschreibe die Anweisung bei der Adresse aus dem 3. Schritt mit einer Jump If Anweisung wo `not` wahr ist, das Konditionsregister das Result Register aus dem 2. Schritt ist, und die Jump Adresse die Adresse aus dem 7. Schritt ist
##### Return
```rust
Statement::Return(expr) => {
	let src = expr.compile(compiler)?;
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::Return { src: Some(src) }, pos);
	Ok(Some(src))
}
```
1. Compiliere die Expression und merke dir den Result Register
2. Schreibe eine Return Anweisung in den jetzigen Closure mit Result Register aus dem 1. Schritt
3. Gebe das Result Register wieder
#### Expression
```rust
impl Compilable for Located<Expression> {
    type Output = Register;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        let Located { value: expr, pos } = self;
        match expr {
            ...
        }
    }
}
```
Bei der Expression muss immer ein Register wiedergegeben werden.
##### Atom
```rust
Expression::Atom(atom) => Located::new(atom, pos).compile(compiler),
```
Einfache Compilierung von dem innerem Atom
##### Binary
```rust
Expression::Binary { op, left, right } => {
	let dst = compiler.frame_mut().new_register();
	let left = left.compile(compiler)?;
	let right = right.compile(compiler)?;
	let op = op.into();
	compiler.frame_mut().closure.write(
		ByteCode::Binary {
			op,
			dst,
			left,
			right,
		},
		pos,
	);
	Ok(dst)
}
```
1. Erstelle ein neues Result Register für die Operation
2. Compiliere die linke Seite
3. Compiliere die rechte Seite
4. Wandele den Binary Operator in seine entsprechende Binary Operation um
5. Schreibe die Binary Anweisung in den jetzigen Closure
6. Gebe das Result Register aus dem 1. Schritt wieder
##### Unary
```rust
let dst = compiler.frame_mut().new_register();
let src = right.compile(compiler)?;
let op = op.into();
compiler
	.frame_mut()
	.closure
	.write(ByteCode::Unary { op, dst, src }, pos);
Ok(dst)
```
1. Erstelle ein neues Result Register für die Operation
3. Compiliere die rechte Seite
4. Wandele den Unary Operator in seine entsprechende Unary Operation um
5. Schreibe die Unary Anweisung in den jetzigen Closure
6. Gebe das Result Register aus dem 1. Schritt wieder
##### Call
```rust
Expression::Call { head, args } => {
	let dst = compiler.frame_mut().new_register();
	let func = head.compile(compiler)?;
	let args_len = args.len() as u8;
	let offset = compiler.frame().registers;
	compiler.frame_mut().registers += args_len as Register;
	for (reg, arg) in (offset..offset + args_len as Register).zip(args) {
		let pos = arg.pos.clone();
		let src = arg.compile(compiler)?;
		compiler
			.frame_mut()
			.closure
			.write(ByteCode::Move { dst: reg, src }, pos);
	}
	compiler.frame_mut().closure.write(
		ByteCode::Call {
			func,
			offset,
			args_len,
			dst: Some(dst),
		},
		pos,
	);
	Ok(dst)
}
```
1. Compiliere die aufzurufende Funktion und merke dir dessen Result Register
2. Merke dir das Offset als die jetzige Register Anzahl
3. Addiere die Anzahl an Argumenten auf die Register Anzahl drauf
4. Iteriere über jedes Argument
	1. Compiliere das Argument und speicher das Result Register
	2. Schreibe in den jetzigen Closure eine Move Anweisung von dem Result Register aus dem 4.1 Schritt zu dem entsprechenden Argument Register
5. Schreibe in den jetzigen Closure eine Call Anweisung mit dem Register der Funktion aus dem 1. Schritt, das Offset aus dem 2. Schritt, die Argumenten Länge, und mit einem leeren Result Register.
#### Atom
```rust
impl Compilable for Located<Atom> {
    type Output = Register;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        let Located { value: atom, pos } = self;
        match atom {
            ...
        }
    }
}
```
Bei dem Atom muss immer ein Register wiedergegeben werden.
##### Expression
```rust
Atom::Expression(expr) => expr.compile(compiler),
```
Compiliere einfach die Expression
##### Ident
```rust
Atom::Ident(ident) => Ok(if let Some(reg) = compiler.frame_mut().local(&ident) {
	reg
} else {
	let addr = compiler.frame_mut().closure.new_string(ident);
	let dst = compiler.frame_mut().new_register();
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::Global { dst, addr }, pos);
	dst
}),
```
1. Versuche das Register der Variable zu finden
2. Wenn du es gefunden hast: Gebe das Register einfach wieder
3. Wenn du es nicht gefunden hast
	1. Lege eine neue String Konstante diese Identifiers an
	2. Erstelle ein neues Register
	3. Schreibe die Global Anweisung in den jetzigen Closure mit der String Adresse aus dem 2.1 Schritt und dem Register aus dem 2.2 Schritt
##### Number und String
```rust
Atom::Number(number) => {
	let addr = compiler.frame_mut().closure.new_number(number);
	let dst = compiler.frame_mut().new_register();
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::Number { dst, addr }, pos);
	Ok(dst)
}
Atom::String(string) => {
	let addr = compiler.frame_mut().closure.new_string(string);
	let dst = compiler.frame_mut().new_register();
	compiler
		.frame_mut()
		.closure
		.write(ByteCode::String { dst, addr }, pos);
	Ok(dst)
}
```
1. Lege eine neue Konstante an
2. Erstelle ein neues Register
3. Schreibe eine Number/String Anweisung in den jetzigen Closure mit der Konstanten Adresse aus dem 1. Schritt und dem Register aus dem 2. Schritt
### In einer Funktion
Das ganze packe ich noch in eine generische Funktion.
```rust
pub fn compile<A: Compilable>(ast: A) -> Result<A::Output, A::Error> {
    ast.compile(&mut Compiler::default())
}
```
`A` ist hier alles was Compiliert werden kann. Die Funktion ist eigentlich nur eine Übersetzung und benutzen tut man sie eigentlich nur mit dem Chunk AST (z.B. `compile(chunk)` wo `chunk` von Typ `Located<Chunk>` ist), aber ist generell eine gute Abstraktion um die Erwartung des Sprach-Level von Betrachtern zu senken.