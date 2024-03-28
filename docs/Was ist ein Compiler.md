Ein Compiler einer Programmiersprachen hat an sich nur eine Funktion: __Text in Computer-Anweisungen umzuwandeln.__ Hierbei gibt es jedoch einen starken unterschied zwischen compilierten und interpretierten Programmiersprachen. Der Hauptunterschied ist, dass Source Code compilierter Sprachen in für die __CPU verständliche Anweisungen__ umgewandelt wird und meist in einer __neuen Datei gespeichert__ wird, dennoch bei interpretierten Sprachen der Source Code lediglich __von einem Interpreten ausgeführt__ und nie in tatsächliche CPU Anweisungen umgeschrieben wird. Der Interpreter hat dabei meist seine __eigenen Anweisungen (Byte Codes)__ die spezifisch für die Sprache oder Sprachgruppen gedacht sind. Wichtig ist zu verstehen, dass der Interpreter Byte Code einen Interpreter __braucht__, da er nicht einfach auf der CPU eines Systems laufen kann. Das heißt diese Interpreter müssen erst einmal __in einer anderen Sprache geschrieben werden__ *(in meinem Fall habe ich Rust gewählt, aber man kann diese in jeder beliebigen Sprache schreiben)*.
Beide Arten von Programmiersprachen haben ihre nach und Vorteile:

| Programmiersprachen Art | Pros                                                                                                                                                                         | Cons                                                                                                                                                                                              |
| :---------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Interpretiert           | Sofortige Ausführung des Programms (Kein Compilationsschritt)<br><br>Kein tiefes Verständnis für Computer nötig<br><br>Dynamische Variablen                                  | Externer Interpret muss runtergeladen werden<br><br>Die Performance ist schlechter (abhängig von der Implementierung)                                                                             |
| Compiliert              | Kann direkt auf dem und gleichen CPUs ausgeführt werden ohne das Compiler Programm<br><br>Der Code ist so schnell wie die CPU<br><br>Mehr Kontrolle über die Speichernutzung | Programme müssen auf verschiedenen CPUs verschieden compiliert werden (wichtig für Multi-Plattform Applikationen)<br><br>Nach ändern des Source Codes muss dieser jedes mal neu compiliert werden |
Beide Arten der Programmiersprachen Implementierung haben ihre Vor- und Nachteile und müssen deswegen für __das Ziel der Programmiersprache__ ausgewählt werden.
Für __meine Implementierung__ einer einfachen Programmiersprache habe ich mich für einen __Interpreter__ entschieden, weil Compiler ohne ein gutes Verständnis von Computern schwer ist zu verstehen.