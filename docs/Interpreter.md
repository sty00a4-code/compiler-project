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
Der Interpreter hat noch zusätzlich eine Globale Umgebung (also eine Tabelle mit Identifier-Value Paaren). Ein Call Frame muss mit einer Referenz zu einem Closure natürlich auch einen Instruction Pointer haben, der zeigt wo im Code der Frame sich gerade befindet, den oben genannten Register Stack, wo ein Register eigentlich nur eine Zellen-Referenz zu einem Wert ist, und noch ein optionales Result Register (das Zielregister aus dem vorherigen Call Frame).
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
Mehr hat die Sprache noch nicht. Für ein gute Programmiersprache fehlt definitiv noch eine Art von Kollektionstypen, wie eine Liste und/oder eine Art Objekt, auch Dictionary in Python oder Table in Lua genannt. Aber das hier ist nur eine Beispiel Sprache, also habe ich diese jetzt erst mal raus gelassen.