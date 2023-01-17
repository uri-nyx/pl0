# PL/0

This is a tiny project-exercise about parsers, compilers and interpreters. It implements a simple descent recursive parser and a compiler for the teaching programming language [https://en.wikipedia.org/wiki/PL/0](PL/0). It emits assembly code directly while parsing for a fantasy computer, the [https://github.com/uri-nyx/ultima](Taleä Computer System). It is highly unoptimized, but it works!

It takes the input from `stdin`, and prints it to `stdout`, but is easily redirectable to files. To assemble, use [https://github.com/hlorenzi/customasm](customasm) (though I am currently using the fork by [https://github.com/JosephAbbey/customasm](JosephAbbey), it **won't assemble with the original**).

The programs rely in a minimal runtime, `crt0.asm`, that provides the intrinsics for input and output, some interrupt and exception handling, and initialization. All this is assembled statically to a binary file, that can be used as rom in the emulator for the Taleä System.

## Grammar

The grammar for PL/0 is this (according to Wkipedia)

```ebnf
program = block "." ;

block = [ "const" ident "=" number {"," ident "=" number} ";"]
        [ "var" ident {"," ident} ";"]
        { "procedure" ident ";" block ";" } statement ;

statement = [ ident ":=" expression | "call" ident 
              | "?" ident | "!" expression | "readchar" ident | "echo" expression
              | "begin" statement {";" statement } "end" 
              | "if" condition "then" statement 
              | "while" condition "do" statement ];

condition = "odd" expression |
            expression ("="|"#"|"<"|"<="|">"|">=") expression ;

expression = [ "+"|"-"] term { ("+"|"-") term};

term = factor {("*"|"/") factor};

factor = ident | number | "(" expression ")";
```
All keywords are **case insensitive**, but identifiers are **case sensitive**.
This version is a bit different from the specification: it needs a `main` procedure, that will be the entry point to the program. Here is a sample:

```pascal
procedure main;
begin
    call factorial
end;
.
```

Notice that it **must** end with te sequence `end; .` if the procedure has a `begin statement`. Alternatively to `?` and `!` for I/O, `read` and `readchar` can be used instead to read integers and characters respectively. `write` and `writechar` can be used as well to output.
