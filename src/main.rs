use std::io;
use std::io::Read;
use std::process;


fn abort(msg: &str) {
    eprintln!("{}", msg);
    process::exit(1);
}

// Recursive descent parser and compiler for PL/0
mod lexer {
    // The lexer categorizes a program into tokens
    // and feeds them to the parser

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Point,
        Comma,
        Semic,
        LParen,
        RParen,
        Equals,
        CEquals,
        Question,
        Bang,
        WriteChar,
        ReadChar,
        Into,
        Hash,
        Less,
        LessEq,
        Great,
        GreatEq,
        Plus,
        Minus,
        Times,
        Slash,
        Const,
        Var,
        Procedure,
        Call,
        Begin,
        End,
        If,
        Then,
        While,
        Do,
        Odd,
        Number{val: i32},
        Ident{val: String}
    }

    impl Token {
        fn from_str(input: &str) -> Token {
            let lowercase = input.to_lowercase();
            match lowercase.as_str() {
                "."=> Token::Point,
                ","=> Token::Comma,
                ";"=> Token::Semic,
                "("=> Token::LParen,
                ")"=> Token::RParen,
                "="=> Token::Equals,
                ":="=> Token::CEquals,
                "?"=> Token::Question,
                "read"=> Token::Question,
                "!"=> Token::Bang,
                "write" => Token::Bang,
                "echo" => Token::WriteChar,
                "writechar" => Token::WriteChar,
                "readchar" => Token::ReadChar,
                "into" => Token::Into,
                "#"=> Token::Hash,
                "<"=> Token::Less,
                "<="=> Token::LessEq,
                ">"=> Token::Great,
                ">="=> Token::GreatEq,
                "+"=> Token::Plus,
                "-"=> Token::Minus,
                "*"=> Token::Times,
                "/"=> Token::Slash,
                "const"=> Token::Const,
                "var"=> Token::Var,
                "procedure"=> Token::Procedure,
                "call"=> Token::Call,
                "begin"=> Token::Begin,
                "end"=> Token::End,
                "if"=> Token::If,
                "then"=> Token::Then,
                "while"=> Token::While,
                "do"=> Token::Do,
                "odd"=> Token::Odd,
                _ => match input.parse::<i32>() {
                    Ok(n) => Token::Number{val: n},
                    Err(_) => Token::Ident{val: input.to_string()}
                }
            }
        }
    }

    pub fn tokenize(source: String) -> Vec<(Token, usize, usize)> {
        let mut tokens: Vec<(Token, usize, usize)> = Vec::new();
        let symbols = [".", ",", ";","(",")","?","!","#","+","-","*","/","=","<",">"];

        let mut source = source;
        for symbol in symbols.iter() {
            source = source.replace(symbol, &format!(" {symbol} "));
        }
        for symbol in [">  =","<  =",": =", "/  /"].iter() {
            let first = symbol.chars().nth(0).unwrap();
            let last = symbol.chars().last().unwrap();
            source = source.replace(symbol, &format!("{first}{last}"));
        }

        for (lineno, line) in source.lines().enumerate() {
            let mut col = 0;
            for word in line.split_whitespace() {
                if word == "//" { break } // Skip comments
                tokens.push((Token::from_str(word), lineno + 1, col)); // start lines from 1
                col += word.len()
            }
        }

        return tokens
    }
}

mod parser {

    // The parser constructs a syntax tree of the program
    use super::lexer::Token;
    pub struct Scanner {
        nesting: usize,
        cursor: usize,
        tokens: Vec<Token>,
        pos: Vec<(usize, usize)>,
        constants: Vec<String>,
        local_constants: usize,
        pub scope_name: String,
        pub scope: Vec<String>,
        pub indentation: String
    }

    impl Scanner {
        pub fn new(tokens: Vec<(Token, usize, usize)>, indentation: String) -> Self {
            let mut pos = Vec::new();
            let mut tokens_only = Vec::new();
            for (tok, lineno, tokno) in tokens {
                tokens_only.push(tok);
                pos.push((lineno, tokno))
            }

            Self {
                nesting: 0,
                cursor: 0,
                tokens: tokens_only,
                pos,
                constants: vec![],
                local_constants: 0,
                scope_name: "global".to_string(),
                scope: vec![],
                indentation
            }
        }

        pub fn search(&self, id: String) -> Result<String, String> {
            let mut scopes: Vec<&str> = self.scope_name.split(".").collect();

            while scopes.len() > 0 {
                let name = format!("{n}.{id}", n = scopes.join("."));
                if self.scope.contains(&name) {
                    return Ok(name);
                } else {
                    _ = scopes.pop();
                }
            }

            if self.scope.contains(&id) {
                Ok(id)
            } else {
                Err(format!("Error {:?}: {id} not found in current scope", self.pos[self.cursor()]))
            }
        }

        pub fn search_const(&self, id: String) -> Result<String, String> {
            let mut scopes: Vec<&str> = self.scope_name.split(".").collect();

            while scopes.len() > 0 {
                let name = format!("{n}.{id}", n = scopes.join("."));
                if self.constants.contains(&name) {
                    return Ok(name);
                } else {
                    _ = scopes.pop();
                }
            }

            if self.scope.contains(&id) {
                Ok(id)
            } else {
                Err(format!("Error {:?}: {id} not found in current scope", self.pos[self.cursor()]))
            }
        }

        pub fn scope_drop(&mut self, until: String) {
            while self.scope.last() != Some(&until) {
                self.scope.pop();
            }

            let l = self.constants.len();
            self.constants.drain((l - self.local_constants)..);
            self.local_constants = 0;
        }

        pub fn emit(&self, asm: &mut Vec<String>, s: String) {
            asm.push(format!("{indentation}{s}", indentation = self.indentation.repeat(self.nesting)))
        }

        pub fn cursor(&self) -> usize {
            self.cursor
        }

        pub fn peek(&self) -> Option<&Token> {
            self.tokens.get(self.cursor)
        }

        pub fn is_match(&self, t: Token) -> bool {
            self.peek() == Some(&t)
        }

        pub fn is_done(&self) -> bool {
            self.cursor() == self.tokens.len()
        }

        pub fn pop(&mut self) -> Option<&Token> {
            match self.tokens.get(self.cursor) {
                None => None,
                Some(t) => {
                    self.cursor += 1;
                    Some(&t)
                }
            }
        }

        pub fn expect(&mut self, target: &Token) -> Result<&Token, String> {
            if self.peek() == Some(target) {
                Ok(self.pop().unwrap())
            } else {
                let tok = format!("{:?}", self.pop());
                let pos = self.pos.get(self.cursor());
                Err(format!("Syntax error {:?}: expected {:?}, got {:?}", pos, target, tok))
            }
        }

        pub fn expect_ident(&mut self) -> Result<String, String> {
            match self.pop().unwrap() {
                Token::Ident{val} => Ok(val.clone()),
                _ => {
                    let pos = self.pos[self.cursor];
                    let tok = self.pop();
                    Err(format!("Syntax error {:?}: Expected identifier, got {:?}", pos, tok.unwrap()))
                }
            }
        }

        pub fn expect_num(&mut self) -> Result<i32, String> {
            match self.pop().unwrap() {
                &Token::Number{val} => Ok(val),
                _ => {
                    let pos = self.pos[self.cursor];
                    let tok = self.pop();
                    Err(format!("Syntax error {:?}: Expected Number, got {:?}", pos, tok.unwrap()))
                }
            }
        }

    }

    pub const A: &str = "a2";
    pub const B: &str = "a3";
    pub const T: &str = "a4";


    pub fn compile(indentation: &str, tokens: Vec<(Token, usize, usize)>, asm: &mut Vec<String>) -> Result<Vec<String>, String> {
        let mut scanner = Scanner::new(tokens, indentation.to_owned());
        program(&mut scanner, asm)?;
        Ok(asm.clone())
    }

    fn program(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        block(scanner, asm)?;
        scanner.expect(&Token::Point)?;

        if !scanner.is_done() {
            return Err(format!("Tokens after '.' (POINT)"));
        }

        Ok(())
    }

    /* block = [ "const" ident "=" number {"," ident "=" number} ";"]
        [ "var" ident {"," ident} ";"]
        { "procedure" ident ";" block ";" } statement ; */

    fn block(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {

        if scanner.is_match(Token::Const) { 
            constant(scanner, asm)?; 
        }
        
        if scanner.is_match(Token::Var) {
            variable(scanner, asm)?;
        }

        while scanner.is_match(Token::Procedure) {
            procedure(scanner, asm)?;
        }

        statement(scanner, asm)?;

        Ok(())
    }

    fn constant(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.local_constants += 1;
        scanner.pop();
        let id = scanner.expect_ident()?;
        scanner.expect(&Token::Equals)?;
        let val = scanner.expect_num()?;
        scanner.emit(asm, format!("{n}{id} = {val}", n = ".".repeat(scanner.nesting)));
        scanner.constants.push(format!("{}.{}", scanner.scope_name, id));

        if scanner.is_match(Token::Comma) {
            constant(scanner, asm)?;
        } else if scanner.is_match(Token::Semic) {
            scanner.pop();
        } else {
            scanner.expect(&Token::Semic)?;
        }
        Ok(())
    }

    fn variable(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        scanner.scope.push(format!("{}.{}", scanner.scope_name, id.clone()));
        let id = format!("{n}{id}", n = ".".repeat(scanner.nesting));
        scanner.emit(asm, format!("#[pragma(var)] {scope}; {id}: #res 4", scope = scanner.scope_name));
        if scanner.is_match(Token::Comma) {
            variable(scanner, asm)?;
        } else if scanner.is_match(Token::Semic) {
            scanner.pop();
        } else {
            scanner.expect(&Token::Semic)?;
        }

        Ok(())
    }

    fn procedure(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        let qualified_id = format!("{}.{}", scanner.scope_name, id.clone());
        scanner.scope_name = qualified_id.clone();
        scanner.scope.push(scanner.scope_name.clone());
        let id = format!("{n}{id}", n = ".".repeat(scanner.nesting));
        scanner.emit(asm, format!("{id}:"));
        scanner.nesting += 1;
        scanner.expect(&Token::Semic)?;

        block(scanner, asm)?;

        scanner.expect(&Token::Semic)?;

        //scanner.emit(asm, format!("pop ra, sp")); // Must be already in RA
        scanner.emit(asm, format!("jalr zero, 0(ra)"));
        scanner.nesting -= 1;
        scanner.scope_drop(qualified_id);
        Ok(())
    }

    /*statement = [ ident ":=" expression | "call" ident 
              | "?" ident | quaero ident | "!" expression | "echo" expression //TODO: corregir README
              | "begin" statement {";" statement } "end" 
              | "if" condition "then" statement 
              | "while" condition "do" statement ];
    */

    fn statement(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        match scanner.peek() {
            Some(Token::Ident{..}) => assignement(scanner, asm),
            Some(Token::Call) => call(scanner, asm),
            Some(Token::Question) => input(scanner, asm),
            Some(Token::Bang) => output(scanner, asm),
            Some(Token::WriteChar) => output_char(scanner, asm),
            Some(Token::ReadChar) => input_char(scanner, asm),
            Some(Token::Begin) => begin(scanner, asm),
            Some(Token::If) => if_statement(scanner, asm),
            Some(Token::While) => while_statement(scanner, asm),
            _ => return Ok(())
        }
    }

    fn assignement(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        let id = scanner.expect_ident()?;
        let id = scanner.search(id)?;
        scanner.expect(&Token::CEquals)?;
        expression(scanner, asm)?;
        if scanner.constants.contains(&id) {
            return Err(format!("Error {:?}: Cannot assign value to constant", scanner.pos[scanner.cursor()]));
        }
        scanner.emit(asm, format!("ssw {A}, {id}, {T}"));
        Ok(())
    }

    fn call(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        let id = scanner.search(id)?.replace("global.", "");
        scanner.emit(asm, format!("push ra, sp"));
        scanner.emit(asm, format!("jal ra, {id}"));
        scanner.emit(asm, format!("pop ra, sp"));
        Ok(())
    }

    fn input(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        if scanner.is_match(Token::Into) {scanner.pop();}
        let id = scanner.search(id)?;
        scanner.emit(asm, format!("push ra, sp"));
        scanner.emit(asm, format!("jal ra, PL0_INPUT.int"));
        scanner.emit(asm, format!("pop ra, sp"));
        scanner.emit(asm, format!("mv {A}, a0"));
        scanner.emit(asm, format!("ssw {A}, {id}, {T}"));
        Ok(())
    }

    fn output(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        expression(scanner, asm)?;
        scanner.emit(asm, format!("mv a0, {A}"));
        scanner.emit(asm, format!("push ra, sp"));
        scanner.emit(asm, format!("jal ra, PL0_OUTPUT"));
        scanner.emit(asm, format!("pop ra, sp"));
        Ok(())
    }

    fn output_char(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        expression(scanner, asm)?;
        scanner.emit(asm, format!("sbd {A}, T_TX(zero)"));
        Ok(())
    }

    fn input_char(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        let id = scanner.search(id)?;
        if scanner.is_match(Token::Into) { scanner.pop(); }
        scanner.emit(asm, format!("push ra, sp"));
        scanner.emit(asm, format!("jal ra, PL0_INPUT.char"));
        scanner.emit(asm, format!("pop ra, sp"));
        scanner.emit(asm, format!("mv {A}, a0"));
        scanner.emit(asm, format!("ssw {A}, {id}, {T}"));
        Ok(())
    }

    fn begin(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> { 
        scanner.pop();
        scanner.emit(asm, format!("{n}begin:", n = ".".repeat(scanner.nesting)));
        scanner.nesting += 1;
        statement(scanner, asm)?;

        while scanner.is_match(Token::Semic) {
            scanner.pop();
            statement(scanner, asm)?;
        }

        scanner.expect(&Token::End)?;
        scanner.nesting -= 1;
        scanner.emit(asm, format!("{n}end:", n = ".".repeat(scanner.nesting)));
        Ok(())
    }

    fn if_statement(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        condition(scanner, asm)?;
        scanner.emit(asm, format!("beq {A}, zero, {n}false_label", n = ".".repeat(scanner.nesting)));
        scanner.expect(&Token::Then)?;
        statement(scanner, asm)?;
        scanner.emit(asm, format!("{n}false_label:", n = ".".repeat(scanner.nesting)));
        Ok(())
    }

    fn while_statement(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        scanner.emit(asm, format!("{n}while:", n = ".".repeat(scanner.nesting)));
        scanner.nesting += 1;

        condition(scanner, asm)?;

        scanner.emit(asm, format!("beq {A}, zero, {n}end_while", n = ".".repeat(scanner.nesting - 1)));

        scanner.expect(&Token::Do)?;
        statement(scanner, asm)?;

        scanner.nesting -= 1;
        scanner.emit(asm, format!("j {n}while", n = ".".repeat(scanner.nesting)));
        scanner.emit(asm, format!("{n}end_while:", n = ".".repeat(scanner.nesting)));
        Ok(())
    }


    /*condition = "odd" expression |
            expression ("="|"#"|"<"|"<="|">"|">=") expression ;*/

    fn compare(scanner: &mut Scanner) -> Result<String, String> {
        let lhs = B; 
        let rhs = A;
        let t = scanner.indentation.repeat(scanner.nesting);
        match scanner.pop() { 
            Some(Token::Equals) => Ok(format!("xor {A}, {lhs}, {rhs}\n{t}sltiu {A}, {A}, 1")),
            Some(Token::Hash) => Ok(format!("xor {A}, {lhs}, {rhs}\n{t}sltu {A}, zero, {A}")),
            Some(Token::Less) => Ok(format!("slt {A}, {lhs}, {rhs}")),
            Some(Token::LessEq) => Ok(format!("slt {A}, {rhs}, {lhs}\n{t}xori {A}, {A}, 1")),
            Some(Token::Great) => Ok(format!("slt {A}, {rhs}, {lhs}")),
            Some(Token::GreatEq) => Ok(format!("slt {A}, {lhs}, {rhs}\n{t}xori {A}, {A}, 1")),
            _ => Err(format!("Syntax Error {:?}: Expected relational operator (for comparison)", scanner.pos[scanner.cursor()])),
        }
    }

    fn condition(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        if scanner.is_match(Token::Odd) {
            scanner.pop();
            expression(scanner, asm)?;
            scanner.emit(asm, format!("andi {A}, {A}, 1"));
        } else {
            expression(scanner, asm)?;
            scanner.emit(asm, format!("push {A}, sp"));
            let comparison= compare(scanner)?;
            expression(scanner, asm)?;
            scanner.emit(asm, format!("pop {B}, sp"));
            scanner.emit(asm, comparison);
        }
        Ok(())
    }

    /*expression = [ "+"|"-"] term { ("+"|"-") term};*/

    fn add_sub(scanner: &mut Scanner) -> Result<String, String> {
        let pos = scanner.pos[scanner.cursor];
        match scanner.pop().unwrap() {
            Token::Plus => Ok(format!("add {A}, {A}, {B}")),
            Token::Minus => Ok(format!("sub {A}, {B}, {A}")),
            tok => Err(format!("Syntax Error {:?}: Expected + or -, got {:?}.", pos, tok))
        }
    }

    fn expression(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        if scanner.is_match(Token::Plus) {
            scanner.pop();
            term(scanner, asm)?;
        }
        else if scanner.is_match(Token::Minus) {
            scanner.pop();
            term(scanner, asm)?;
            scanner.emit(asm, format!("not {A}, {A}")); // Maybe would be handy allow to add an immediate here
            scanner.emit(asm, format!("addi {A}, {A}, 1"));
        } else {
            term(scanner, asm)?;
        }

        while scanner.is_match(Token::Plus) || scanner.is_match(Token::Minus) {
            scanner.emit(asm, format!("push {A}, sp"));
            let expression = add_sub(scanner)?;
            term(scanner, asm)?;
            scanner.emit(asm, format!("pop {B}, sp"));
            scanner.emit(asm, expression);
        }

        Ok(())
    }

    /*term = factor {("*"|"/") factor};*/

    fn mul_div(scanner: &mut Scanner) -> Result<String, String> {
        let pos = scanner.pos[scanner.cursor];

        match scanner.pop().unwrap() {
            Token::Times => Ok(format!("mul zero, {A}, {A}, {B}")),
            Token::Slash => Ok(format!("idiv {A}, zero, {B}, {A}")),
            tok => Err(format!("Syntax Error {:?}: Expected * or /, got {:?}.", pos, tok))
        }
    }
    
    fn term(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        factor(scanner, asm)?;
        while scanner.is_match(Token::Times) || scanner.is_match(Token::Slash) {
            scanner.emit(asm, format!("push {A}, sp"));
            let term = mul_div(scanner)?;
            factor(scanner, asm)?;
            scanner.emit(asm, format!("pop {B}, sp"));
            scanner.emit(asm, term);
        }
        Ok(())
    }

    /*factor = ident | number | "(" expression ")";*/

    fn factor(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        match scanner.peek() {
            Some(Token::Ident{..}) => {
                let id = scanner.expect_ident()?;
                match scanner.search(id.clone()) {
                    Ok(id) => scanner.emit(asm, format!("llw {A}, {id}")),
                    Err(_) => {
                        let id = scanner.search_const(id)?.replace("global.", "");
                        scanner.emit(asm, format!("li {A}, {id}"))
                    }
                }
                Ok(())
            },
            Some(Token::Number{..}) => {
                let num = scanner.expect_num()?;
                scanner.emit(asm, format!("li {A}, {num}"));
                Ok(())
            },
            Some(Token::LParen) => {
                scanner.pop();
                expression(scanner, asm)?;
                scanner.expect(&Token::RParen)?;
                Ok(())
            },
            _ => Err(format!("Error {:?}: Expected Factor", scanner.pos[scanner.cursor()]))
        }

    }

}

fn main() -> io::Result<()> {
    use crate::parser::*;
    use crate::lexer::tokenize;

    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;

    let source = tokenize(source);
    let mut compiled = vec![];

    match compile("\t", source, &mut compiled) {
        Ok(code) => compiled = code,
        Err(err) => {
            abort(&err);
            return Ok(())
        }
    }



    let mut text = vec![];
    let mut data = vec![];
    for line in compiled {
        if line.contains("#[pragma(var)]") {
            data.push(line.replace("#[pragma(var)]", ""));
        } else {
            text.push(line);
        }
    }

    let mut variables = vec![];
    for line in data {
        let (scope, name) = line.split_once(";").clone().unwrap();
        variables.push((scope.trim().to_owned(), name.trim().to_owned()));
    }

    variables.sort_by_key(|decl| {
        let scope = decl.0.clone();
        let scope: Vec<&str> = scope.split(".").collect();
        scope.len()
    });
    let mut var_table = vec!["global:".to_owned()];
    let mut current_scope = String::new();
    let mut nesting = 0;
    for (scope, name) in variables {
        if scope != current_scope {
            current_scope = scope.clone();
            let s: Vec<&str> = scope.split(".").collect();
            nesting = s.len() - 1;
            let s = format!("{t}{n}{s}:", t="\t".repeat(nesting), n=".".repeat(nesting), s=s.last().unwrap());
            var_table.push(s);
        }

        if scope == "global" {
            var_table.push(format!("\t.{name}"));
            continue;
        }

        var_table.push(format!("\t{t}{n}{name}",  t="\t".repeat(nesting), n=".".repeat(nesting)))
    }
    
    println!("#include \"std/crt0.asm\"");
    println!("; section TEXT --------");

    for line in text {
        println!("{}", line);
    }
    println!("");
    println!("; ENTRY POINT -------");
    println!("Start: ");
    println!("\tmv {A}, zero");
    println!("\tmv {B}, zero");
    println!("\tmv {T}, zero");

    println!("\tjal ra, main");
    println!("\tj crt0.exit");
    println!("");
    println!("; section DATA --------");
    println!("\t#align 32");
    println!("; varialbes -------");
    println!("global:");
    for line in var_table {
        if line == "global:".to_owned() { continue }
        println!("{}", line);
    }
    println!("; ----------------");
    println!("\t\t#align 32");
    println!("\t\t#res 1024");
    println!("\t\t#align 32");
    println!("\t.stack:");
    println!("\t\t#res 10");
    println!("\t\t#align 32");
    println!("\t.out_buff:");
    println!("\t\t#res 10");
    println!("\t\t#align 32");
    println!("\t.in_buff:");
    println!("\t\t#res 10");
    println!("\t\t#align 32");




    Ok(())
}