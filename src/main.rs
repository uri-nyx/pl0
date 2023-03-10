use std::io;
use std::io::Read;
use std::process;
//TODO: change comments to Pascal Like?`{}`

fn abort(msg: &str) {
    eprintln!("{}", msg);
    process::exit(1);
}

// Recursive descent parser and compiler for PL/0
mod lexer {
    // The lexer categorizes a program into tokens
    // and feeds them to the parser

    #[derive(Debug, PartialEq, Clone)]
    pub enum Token {
        // DELIMITERS
        Point,
        Comma,
        Semic,
        LParen,
        RParen,
        LBrack,
        RBrack,

        // OPERATORS
        Equals,
        CEquals,
        Hash,
        Less,
        LessEq,
        Great,
        GreatEq,
        Plus,
        Minus,
        Times,
        Slash,
        Odd,
        Not,
        And,
        Or,
        Mod,

        // IO
        Question,
        Bang,
        WriteChar,
        ReadChar,
        WriteStr,
        Into,

        // KEYWORDS
        Const,
        Var,
        Procedure,
        Forward,
        Call,
        Begin,
        End,
        If,
        Then,
        Else,
        While,
        Do,
        Size,
        Exit,
        Str(String),

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
                "["=> Token::LBrack,
                "]"=> Token::RBrack,
                "?"=> Token::Question,
                "read"=> Token::Question,
                "!"=> Token::Bang,
                "write" => Token::Bang,
                "writeint" => Token::Bang,
                "echo" => Token::WriteChar,
                "writechar" => Token::WriteChar,
                "readchar" => Token::ReadChar,
                "writestr" => Token::WriteStr,
                "into" => Token::Into,
                "odd"=> Token::Odd,
                "="=> Token::Equals,
                ":="=> Token::CEquals,
                "#"=> Token::Hash,
                "<>" => Token::Hash,
                "<"=> Token::Less,
                "<="=> Token::LessEq,
                ">"=> Token::Great,
                ">="=> Token::GreatEq,
                "+"=> Token::Plus,
                "-"=> Token::Minus,
                "*"=> Token::Times,
                "/"=> Token::Slash,
                "exit" => Token::Exit,
                "not" => Token::Not,
                "and" => Token::And,
                "or" => Token::Or,
                "mod" => Token::Mod,
                "const"=> Token::Const,
                "var"=> Token::Var,
                "procedure"=> Token::Procedure,
                "forward" => Token::Forward,
                "call"=> Token::Call,
                "begin"=> Token::Begin,
                "end"=> Token::End,
                "if"=> Token::If,
                "then"=> Token::Then,
                "else"=> Token::Else,
                "while"=> Token::While,
                "do"=> Token::Do,
                "size"=> Token::Size,
                _ => match input.parse::<i32>() {
                    Ok(n) => Token::Number{val: n},
                    Err(_) => Token::Ident{val: input.to_string()}
                }
            }
        }
    }

    pub fn tokenize(source: String) -> Vec<(Token, usize, usize)> {
        let mut tokens: Vec<(Token, usize, usize)> = Vec::new();
        let symbols = [".", ",", ";","(",")","?","!","#","+","-","*","/","=","<",">", "[", "]", "{", "}"];
        let mut source = source;
        let mut strings = vec![];
        let mut current = String::new();
        let mut recording = false;
        for c in source.chars() {
            if c == '\'' {
                if recording {
                    strings.push(current);
                    current = String::new();
                }
                recording = !recording;
            } else if recording {
                current.push(c);
            }
        }

        for (i, string) in strings.iter().enumerate() {
            source = source.replace(&format!("'{string}'"), &format!("___pl0__reserved__identifier__string__{i}"));
        }

        for symbol in symbols.iter() {
            source = source.replace(symbol, &format!(" {symbol} "));
        }
        for symbol in [">  =","<  =",": =", "/  /", "<  >"].iter() {
            let first = symbol.chars().nth(0).unwrap();
            let last = symbol.chars().last().unwrap();
            source = source.replace(symbol, &format!("{first}{last}"));
        }

        let mut is_comment = false;
        for (lineno, line) in source.lines().enumerate() {
            let mut col = 0;
            for word in line.split_whitespace() {
                match word {
                    "//" => break, // Skip line comments
                    "{"  => {
                            is_comment = true;
                            col += 1;
                    }
                    "}"  => {
                        is_comment = false;
                        col += 1;
                    }
                    _ => {
                        if !is_comment {
                            tokens.push((Token::from_str(word), lineno + 1, col)); // start lines from 1
                            col += word.len()
                        } else {
                            col += word.len()
                        }
                    }
                }
            }
        }

        let mut final_tokens = vec![];
        for tok in &tokens {
            match &tok.0 {
                Token::Ident { val } => {
                    if val.contains("___pl0__reserved__identifier__string__") {
                        let i = val.split("___pl0__reserved__identifier__string__").last().unwrap();
                        let i = i.parse::<usize>().unwrap();
                        let string = strings[i].clone();
                        final_tokens.push((Token::Str(string), tok.1, tok.2));
                    } else {
                        final_tokens.push((tok.0.clone(), tok.1, tok.2))
                    }
                },
                _ => final_tokens.push((tok.0.clone(), tok.1, tok.2))
            }
        }

        return final_tokens
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
        arrays: Vec<String>,
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
                arrays: vec![],
                local_constants: 0,
                scope_name: "global".to_string(),
                scope: vec![],
                indentation
            }
        }

        pub fn is_array(&self, id: String) -> Result<(), String> {
            if !self.arrays.contains(&id) {
                return Err(format!("Error {:?}: {id} is not an array", self.pos[self.cursor()]));
            }

            Ok(())
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
        
        if !scanner.is_done() {
            scanner.expect(&Token::Point)?;
            if !scanner.is_done() {
                return Err(format!("Tokens after '.' (POINT)"));
            }
        }

        Ok(())
    }

    /* block = [ "const" ident "=" number {"," ident "=" number} ";"]
        [ "var" ident {"," ident} ";"]
        { "forward" ident ";" }
        { "procedure" ident ";" block ";" } statement ; */

    fn block(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.local_constants = 0;
        if scanner.is_match(Token::Const) { 
            constant(scanner, asm)?; 
        }
        
        if scanner.is_match(Token::Var) {
            variable(scanner, asm)?;
        }

        while scanner.is_match(Token::Forward) {
            forward(scanner)?;
        }

        while scanner.is_match(Token::Procedure) {
            procedure(scanner, asm)?;
        }

        statement(scanner, asm)?;

        Ok(())
    }

    fn constant_val(scanner: &mut Scanner) -> Result<i32, String> {
        let tok = scanner.peek().clone();
        match tok {
            Some(&Token::Number{..}) => Ok(scanner.expect_num()?),
            Some(&Token::Str(ref s)) => {
                let s = s.to_owned();
                if s.len() == 1 {
                    scanner.pop();
                    Ok(s.as_bytes()[0] as i32)
                } else {
                    return Err(format!("Constant Error {:?}: Constants can only take one character strings, got {s}", scanner.pos[scanner.cursor()]))
                }
            }
            tok => return Err(format!("Constant Error {:?}: Constants can only be declared with, a number or a character, {:?}", scanner.pos[scanner.cursor()], tok))
        }
    }

    fn constant(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.local_constants += 1;
        scanner.pop();
        let id = scanner.expect_ident()?;
        scanner.expect(&Token::Equals)?;
        let val = constant_val(scanner)?;

        /*if val.is_none() { //TODO: implement simple arithmetic in constants. Otherwhise this adds no value
            let id = scanner.expect_ident();
            if !id.is_err() {
                let id = scanner.search_const(id)?;
            } else {
                return Err(format!("Constant Error {:?}: Constants can only be declared with constants, number or character", scanner.pos[scanner.cursor()]))
            }
        }*/

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
        let qualified_id = format!("{}.{}", scanner.scope_name, id.clone());
        scanner.scope.push(qualified_id.clone());
        let id = format!("{n}{id}", n = ".".repeat(scanner.nesting));

        if scanner.is_match(Token::Size) {
            scanner.pop();
            match scanner.peek() {
                Some(Token::Ident{..}) => {
                let constant = scanner.expect_ident()?;
                let size = scanner.search_const(constant)?.replace("global.", "");
                scanner.emit(asm, format!("#[pragma(var)] {scope}; {id}: #res {size} * 4", scope = scanner.scope_name));
                scanner.emit(asm, format!("#[pragma(var)] {scope}; {n}.len: #d32 {size}`32", scope = scanner.scope_name, n=".".repeat(scanner.nesting)));
                }
                _ => {
                let size = scanner.expect_num()?;
                if size < 1 { return Err(format!("Error defining array: {id} ({:?}): Array size must be greater than 0", scanner.pos))}
                scanner.emit(asm, format!("#[pragma(var)] {scope}; {id}: #res {size} * 4 ", scope = scanner.scope_name));
                scanner.emit(asm, format!("#[pragma(var)] {scope}; {n}.len: #d32 {size}`32", scope = scanner.scope_name, n=".".repeat(scanner.nesting)));
                }
            }
        scanner.arrays.push(qualified_id);
        } else {
            scanner.emit(asm, format!("#[pragma(var)] {scope}; {id}: #res 4", scope = scanner.scope_name));
        }

        if scanner.is_match(Token::Comma) {
            variable(scanner, asm)?;
        } else if scanner.is_match(Token::Semic) {
            scanner.pop();
        } else {
            scanner.expect(&Token::Semic)?;
        }

        Ok(())
    }

    fn forward(scanner: &mut Scanner) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        let qualified_id = format!("{}.{}", scanner.scope_name, id.clone());
        scanner.scope.push(qualified_id);
        scanner.expect(&Token::Semic)?;
        Ok(())
    }

    fn procedure(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        let id = scanner.expect_ident()?;
        let qualified_id = format!("{}.{}", scanner.scope_name, id.clone());
        let old_scope = scanner.scope_name.clone();
        scanner.scope_name = qualified_id.clone();
        scanner.scope.push(scanner.scope_name.clone());
        let id = format!("{n}{id}", n = ".".repeat(scanner.nesting));
        scanner.emit(asm, format!("{id}:"));
        scanner.nesting += 1;
        scanner.expect(&Token::Semic)?;

        block(scanner, asm)?;

        if scanner.cursor() + 1 == scanner.tokens.len() { 
            scanner.expect(&Token::Point)?; 
        } else {
            scanner.expect(&Token::Semic)?;
        }

        //scanner.emit(asm, format!("pop ra, sp")); // Must be already in RA
        scanner.emit(asm, format!("jalr zero, 0(ra)"));
        scanner.nesting -= 1;
        scanner.scope_drop(qualified_id);
        scanner.scope_name = old_scope;
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
            Some(Token::WriteStr) => output_string(scanner, asm),
            Some(Token::Exit) => exit_statement(scanner, asm),
            _ => return Ok(())
        }
    }

    fn assignement(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        let id = scanner.expect_ident()?;
        let id = scanner.search(id)?;

        if scanner.constants.contains(&id) {
            return Err(format!("Error {:?}: Cannot assign value to constant", scanner.pos[scanner.cursor()]));
        }

        if scanner.is_match(Token::LBrack) {
            scanner.is_array(id.clone())?;
            scanner.pop();
            expression(scanner, asm)?;
            scanner.expect(&Token::RBrack)?;
            scanner.expect(&Token::CEquals)?;
            scanner.emit(asm, format!("la {T}, {id}"));
            scanner.emit(asm, format!("muli {A}, {A}, 4"));
            scanner.emit(asm, format!("add {T}, {A}, {T}"));
            expression(scanner, asm)?;
            scanner.emit(asm, format!("sw {A}, 0({T})"));
            
        } else {
            scanner.expect(&Token::CEquals)?;
            expression(scanner, asm)?;
            scanner.emit(asm, format!("ssw {A}, {id}, {T}"));
        }
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
        if scanner.is_match(Token::Into) {scanner.pop();}
        let id = scanner.expect_ident()?;
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

    fn output_string(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> { //TODO: make strigns be of 8 bits instead of 32
        scanner.pop();
        match scanner.peek() {
            Some(&Token::Ident{..}) => { // Print until \0 even if array is larger
                let id = scanner.expect_ident()?;
                let id = scanner.search(id)?;
                scanner.is_array(id.clone())?;
                scanner.emit(asm, format!("la {A}, {id}"));
                scanner.emit(asm, format!("{n}writeStr_loop:", n=".".repeat(scanner.nesting)));
                scanner.emit(asm, format!("lw {T}, 0({A})"));
                scanner.emit(asm, format!("beq {T}, zero, {n}writeStr_exit", n=".".repeat(scanner.nesting)));
                scanner.emit(asm, format!("sbd {T}, T_TX(zero)"));
                scanner.emit(asm, format!("addi {A}, {A}, 4"));
                scanner.emit(asm, format!("j {n}writeStr_loop", n=".".repeat(scanner.nesting)));
                scanner.emit(asm, format!("{n}writeStr_exit:", n=".".repeat(scanner.nesting)));
            }
            Some(&Token::Str(ref s)) => {
                let s = s.clone();
                scanner.pop();
                let str_id = format!("str_{}_{}", scanner.pos[scanner.cursor()].0, scanner.pos[scanner.cursor()].1);
                scanner.emit(asm, format!("#[pragma(string_litteral)]{str_id}: #d \"{s}\\0\"\n#align 32"));
                scanner.emit(asm, format!("push ra, sp"));
                scanner.emit(asm, format!("la a0, {str_id}"));
                scanner.emit(asm, format!("jal ra, crt0.puts"));
                scanner.emit(asm, format!("pop ra, sp"));
            }
            _ => return Err(format!("Syntax Error ({:?}): WriteStr takes either a string or an array", scanner.pos[scanner.cursor()])),
        }
        Ok(())
    }

    fn input_char(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        if scanner.is_match(Token::Into) { scanner.pop(); }
        let id = scanner.expect_ident()?;
        let id = scanner.search(id)?;
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
        scanner.emit(asm, format!("{n}if:", n = ".".repeat(scanner.nesting)));
        scanner.nesting += 1;
        condition(scanner, asm)?;
        scanner.emit(asm, format!("beq {A}, zero, {n}else", n = ".".repeat(scanner.nesting - 1)));
        scanner.expect(&Token::Then)?;
        statement(scanner, asm)?;
        scanner.nesting -= 1;
        scanner.emit(asm, format!("j {n}exit", n = ".".repeat(scanner.nesting)));
        scanner.emit(asm, format!("{n}else:", n = ".".repeat(scanner.nesting)));
        scanner.nesting += 1;
        if scanner.is_match(Token::Else) { 
            scanner.pop();
            statement(scanner, asm)? 
        }
        scanner.nesting -= 1;
        scanner.emit(asm, format!("{n}exit:", n = ".".repeat(scanner.nesting)));

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

    fn exit_statement(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        scanner.pop();
        expression(scanner, asm)?;
        scanner.emit(asm, format!("mv a0, {A}"));
        scanner.emit(asm, format!("j crt0.exit")); //TODO: add a better exit point
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

    fn add_sub_or(scanner: &mut Scanner) -> Result<String, String> {
        let pos = scanner.pos[scanner.cursor];
        match scanner.pop().unwrap() {
            Token::Plus => Ok(format!("add {A}, {A}, {B}")),
            Token::Minus => Ok(format!("sub {A}, {B}, {A}")),
            Token::Or => Ok(format!("or {A}, {B}, {A}")),
            tok => Err(format!("Syntax Error {:?}: Expected +, -, or 'or', got {:?}.", pos, tok))
        }
    }

    fn expression(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {

        match scanner.peek() {
            Some(&Token::Plus) => {
                scanner.pop();
                term(scanner, asm)?;
            }
            Some(&Token::Minus) => {
                scanner.pop();
                term(scanner, asm)?;
                scanner.emit(asm, format!("not {A}, {A}")); // Maybe would be handy allow to add an immediate here
                scanner.emit(asm, format!("addi {A}, {A}, 1"));
            } 
            Some(&Token::Not) => {
                scanner.pop();
                term(scanner, asm)?;
                scanner.emit(asm, format!("not {A}, {A}"));
            }
            _ =>  term(scanner, asm)?
        }

        while scanner.is_match(Token::Plus) || scanner.is_match(Token::Minus) ||  scanner.is_match(Token::Or) {
            scanner.emit(asm, format!("push {A}, sp"));
            let expression = add_sub_or(scanner)?;
            term(scanner, asm)?;
            scanner.emit(asm, format!("pop {B}, sp"));
            scanner.emit(asm, expression);
        }

        Ok(())
    }

    /*term = factor {("*"|"/") factor};*/

    fn mul_div_mod_and(scanner: &mut Scanner) -> Result<String, String> {
        let pos = scanner.pos[scanner.cursor];

        match scanner.pop().unwrap() {
            Token::Times => Ok(format!("mul zero, {A}, {A}, {B}")),
            Token::Slash => Ok(format!("idiv {A}, zero, {B}, {A}")),
            Token::Mod => Ok(format!("idiv  zero, {A}, {B}, {A}")),
            Token::And => Ok(format!("and {A}, {B}, {A}")),
            tok => Err(format!("Syntax Error {:?}: Expected *, /, 'mod', or 'and' got {:?}.", pos, tok))
        }
    }
    
    fn term(scanner: &mut Scanner, asm: &mut Vec<String>) -> Result<(), String> {
        factor(scanner, asm)?;
        while scanner.is_match(Token::Times) || scanner.is_match(Token::Slash) {
            scanner.emit(asm, format!("push {A}, sp"));
            let term = mul_div_mod_and(scanner)?;
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
                    Ok(id) => {
                        if scanner.is_match(Token::LBrack) {
                            scanner.is_array(id.clone())?;
                            scanner.pop();
                            expression(scanner, asm)?;
                            scanner.expect(&Token::RBrack)?;
                            scanner.emit(asm, format!("muli {A}, {A}, 4"));
                            scanner.emit(asm, format!("la {T}, {id}"));
                            scanner.emit(asm, format!("add {T}, {A}, {T}"));
                            scanner.emit(asm, format!("lw {A}, 0({T})"));
                        }
                        else { scanner.emit(asm, format!("llw {A}, {id}")) }
                        }
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
    let mut string_litterals = vec![];
    for line in compiled {
        if line.contains("#[pragma(var)]") {
            data.push(line.replace("#[pragma(var)]", ""));
        } else if line.contains("#[pragma(string_litteral)]") {
            string_litterals.push(line.replace("#[pragma(string_litteral)]", ""));
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

    println!("\tjal ra, main");//Main exits with 0 implicitly
    println!("\tmv a0, zero"); //TODO: maybe move exit code to a system vvariable
    println!("\tj crt0.exit");
    println!("");
    println!("; section DATA --------");
    println!("; String Litterals-----");
    for line in string_litterals {
        println!("{}", line.trim());
    }
    println!("; Variables -----------");
    println!("global:");
    for line in var_table {
        if line == "global:".to_owned() { continue }
        println!("{}", line);
    }
    println!("; ---------------------");
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