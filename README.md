# PL/0

This is a tiny project-exercise about parsers, compilers and interpreters. It implements a simple descent recursive parser and a compiler for the teaching programming language [https://en.wikipedia.org/wiki/PL/0](PL/0). It emits assembly code directly while parsing for a fantasy computer, the [https://github.com/uri-nyx/ultima](Taleä Computer System). It is highly unoptimized, but it works!

A version for a somewhat *standard* PL/0 compiler can be found in the *standard* branch. This version implements extra features (arrays and strings) to allow self-hosting, as described by [https://briancallahan.net/blog/20210822.html](Brian Callahan), and implements a self hosted PL/0 for the Taleä Computer System (Just adding another backend to [https://github.com/ibara/pl0c](Brian Callahan's compiler)).

It takes the input from `stdin`, and prints it to `stdout`, but is easily redirectable to files. To assemble, use [https://github.com/hlorenzi/customasm](customasm) (though I am currently using the fork by [https://github.com/JosephAbbey/customasm](Joseph Abbey), it **won't assemble with the original**).

The programs rely in a minimal runtime, `crt0.asm`, that provides the intrinsics for input and output, some interrupt and exception handling, and initialization. All this is assembled statically to a binary file, that can be used as rom in the emulator for the Taleä System.

## Grammar

The grammar for PL/0 with extensions, from [https://briancallahan.net/blog/20210906.html](Brian Callahan's blog), this:

```ebnf

program     = block "." .

block       = [ "const" ident "=" number { "," ident "=" number } ";" ]
            [ "var" ident [ array ] { "," ident [ array ] } ";" ]
            { "forward" ident ";" }
            { "procedure" ident ";" block ";" } statement .

statement   = [ ident ":=" expression
            | "call" ident
            | "begin" statement { ";" statement } "end"
            | "if" condition "then" statement [ "else" statement ]
            | "while" condition "do" statement
            | input_int [ "into" ] ident
            | "readchar" [ "into" ] ident
            | output_int expression
            | output_char expression
            | "writeStr" ( ident | string )
            | "exit" expression ] .

condition   = "odd" expression
            | expression ( comparator ) expression .

expression  = [ "+" | "-" | "not" ] term { ( "+" | "-" | "or" ) term } .

term        = factor { ( "*" | "/" | "mod" | "and" ) factor } .

factor      = ident
            | number
            | "(" expression ")" .

comparator  = "=" | "#" | "<" | ">" | "<=" | ">=" | "<>" .
array       = "size" number .
input_int   = "read" | "?".
output_int  = "write" | "!".
output_char = "writechar" | "echo".

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

 end with te sequence `end; .` if the procedure has a `begin statement`. Alternatively to `?` and `!` for I/O, `read` and `readchar` can be used instead to read integers and characters respectively. `write` and `writechar` can be used as well to output. Comments are introduced by `//` and last until the end of the line.
