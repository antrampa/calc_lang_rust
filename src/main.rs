// calc_lang — a tiny programming language for math, written in Rust.
//
// The pipeline every interpreter follows:
//   source text  --(Lexer)-->  tokens  --(Parser)-->  AST  --(Evaluator)-->  value
//
// Supported so far:
//   numbers:        42, 3.14
//   operators:      + - * / % ^   (^ is power, right-associative)
//   parentheses:    (1 + 2) * 3
//   unary minus:    -5, -(2 + 3)
//   variables:      let x = 10      then   x * 2
//   functions:      sqrt(16), abs(-3), sin(x), cos(x), min(a, b), max(a, b)
//   constants:      pi, e

use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead, Write};

// ───────────────────────────── 1. LEXER ─────────────────────────────
// Turns "let x = 1 + 2" into [Let, Ident("x"), Equals, Number(1), Plus, Number(2)]

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Let,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    Equals,
    Eof,
}

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\r' | '\n' => i += 1,
            '#' => break, // comment: ignore rest of line
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text
                    .parse::<f64>()
                    .map_err(|_| format!("invalid number '{text}'"))?;
                tokens.push(Token::Number(n));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                tokens.push(if word == "let" { Token::Let } else { Token::Ident(word) });
            }
            _ => {
                tokens.push(match c {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Star,
                    '/' => Token::Slash,
                    '%' => Token::Percent,
                    '^' => Token::Caret,
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    ',' => Token::Comma,
                    '=' => Token::Equals,
                    other => return Err(format!("unexpected character '{other}'")),
                });
                i += 1;
            }
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

// ───────────────────────────── 2. PARSER ─────────────────────────────
// Builds an Abstract Syntax Tree (AST). "1 + 2 * 3" becomes:
//        Add
//       /   \
//      1    Mul
//          /   \
//         2     3

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug)]
enum Expr {
    Number(f64),
    Var(String),
    Neg(Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
enum Stmt {
    Let(String, Expr),
    Expr(Expr),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

// Recursive descent: one function per precedence level, lowest first.
//   statement  := "let" IDENT "=" expr | expr
//   expr       := term (("+" | "-") term)*
//   term       := unary (("*" | "/" | "%") unary)*
//   unary      := "-" unary | power
//   power      := primary ("^" unary)?          (right-associative)
//   primary    := NUMBER | IDENT | IDENT "(" args ")" | "(" expr ")"
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let t = self.advance();
        if t == expected {
            Ok(())
        } else {
            Err(format!("expected {expected:?}, found {t:?}"))
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        let stmt = if *self.peek() == Token::Let {
            self.advance();
            let name = match self.advance() {
                Token::Ident(n) => n,
                t => return Err(format!("expected variable name after 'let', found {t:?}")),
            };
            self.expect(Token::Equals)?;
            Stmt::Let(name, self.parse_expr()?)
        } else {
            Stmt::Expr(self.parse_expr()?)
        };
        if *self.peek() != Token::Eof {
            return Err(format!("unexpected token {:?}", self.peek()));
        }
        Ok(stmt)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if *self.peek() == Token::Minus {
            self.advance();
            return Ok(Expr::Neg(Box::new(self.parse_unary()?)));
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_primary()?;
        if *self.peek() == Token::Caret {
            self.advance();
            let exp = self.parse_unary()?; // recursion => 2^3^2 = 2^(3^2)
            return Ok(Expr::Binary(Box::new(base), BinOp::Pow, Box::new(exp)));
        }
        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Ident(name) => {
                if *self.peek() == Token::LParen {
                    self.advance();
                    let mut args = Vec::new();
                    if *self.peek() != Token::RParen {
                        args.push(self.parse_expr()?);
                        while *self.peek() == Token::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::LParen => {
                let e = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(e)
            }
            t => Err(format!("unexpected token {t:?}")),
        }
    }
}

// ───────────────────────────── 3. EVALUATOR ─────────────────────────────
// Walks the AST and computes the result.

struct Interpreter {
    vars: HashMap<String, f64>,
}

enum Outcome {
    Value(f64),
    Assigned(String, f64),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Outcome::Value(v) => write!(f, "{}", fmt_num(*v)),
            Outcome::Assigned(n, v) => write!(f, "{n} = {}", fmt_num(*v)),
        }
    }
}

fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl Interpreter {
    fn new() -> Self {
        let mut vars = HashMap::new();
        vars.insert("pi".to_string(), std::f64::consts::PI);
        vars.insert("e".to_string(), std::f64::consts::E);
        Interpreter { vars }
    }

    fn run(&mut self, line: &str) -> Result<Option<Outcome>, String> {
        let tokens = tokenize(line)?;
        if tokens == [Token::Eof] {
            return Ok(None); // empty line or comment
        }
        let stmt = Parser::new(tokens).parse_statement()?;
        match stmt {
            Stmt::Let(name, expr) => {
                let v = self.eval(&expr)?;
                self.vars.insert(name.clone(), v);
                Ok(Some(Outcome::Assigned(name, v)))
            }
            Stmt::Expr(expr) => Ok(Some(Outcome::Value(self.eval(&expr)?))),
        }
    }

    fn eval(&self, expr: &Expr) -> Result<f64, String> {
        match expr {
            Expr::Number(n) => Ok(*n),
            Expr::Var(name) => self
                .vars
                .get(name)
                .copied()
                .ok_or_else(|| format!("unknown variable '{name}'")),
            Expr::Neg(e) => Ok(-self.eval(e)?),
            Expr::Binary(l, op, r) => {
                let (a, b) = (self.eval(l)?, self.eval(r)?);
                match op {
                    BinOp::Add => Ok(a + b),
                    BinOp::Sub => Ok(a - b),
                    BinOp::Mul => Ok(a * b),
                    BinOp::Div if b == 0.0 => Err("division by zero".into()),
                    BinOp::Div => Ok(a / b),
                    BinOp::Mod if b == 0.0 => Err("modulo by zero".into()),
                    BinOp::Mod => Ok(a % b),
                    BinOp::Pow => Ok(a.powf(b)),
                }
            }
            Expr::Call(name, args) => {
                let vals = args
                    .iter()
                    .map(|a| self.eval(a))
                    .collect::<Result<Vec<_>, _>>()?;
                call_builtin(name, &vals)
            }
        }
    }
}

fn call_builtin(name: &str, args: &[f64]) -> Result<f64, String> {
    let one = |f: fn(f64) -> f64| -> Result<f64, String> {
        match args {
            [x] => Ok(f(*x)),
            _ => Err(format!("{name}() takes 1 argument, got {}", args.len())),
        }
    };
    let two = |f: fn(f64, f64) -> f64| -> Result<f64, String> {
        match args {
            [x, y] => Ok(f(*x, *y)),
            _ => Err(format!("{name}() takes 2 arguments, got {}", args.len())),
        }
    };
    match name {
        "sqrt" => match args {
            [x] if *x < 0.0 => Err("sqrt of a negative number".into()),
            _ => one(f64::sqrt),
        },
        "abs" => one(f64::abs),
        "sin" => one(f64::sin),
        "cos" => one(f64::cos),
        "tan" => one(f64::tan),
        "ln" => one(f64::ln),
        "floor" => one(f64::floor),
        "ceil" => one(f64::ceil),
        "round" => one(f64::round),
        "min" => two(f64::min),
        "max" => two(f64::max),
        _ => Err(format!("unknown function '{name}'")),
    }
}

// ───────────────────────────── 4. MAIN: REPL or file ─────────────────────────────

fn main() {
    let mut interp = Interpreter::new();
    let args: Vec<String> = std::env::args().collect();

    // `calc_lang program.calc` runs a file line by line
    if let Some(path) = args.get(1) {
        let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(1);
        });
        for (n, line) in src.lines().enumerate() {
            match interp.run(line) {
                Ok(Some(out)) => println!("{out}"),
                Ok(None) => {}
                Err(e) => {
                    eprintln!("line {}: error: {e}", n + 1);
                    std::process::exit(1);
                }
            }
        }
        return;
    }

    // Otherwise start an interactive REPL
    println!("calc_lang v0.1 — type an expression, or 'exit' to quit");
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 || line.trim() == "exit" {
            break;
        }
        match interp.run(&line) {
            Ok(Some(out)) => println!("{out}"),
            Ok(None) => {}
            Err(e) => println!("error: {e}"),
        }
    }
}

// ───────────────────────────── TESTS (cargo test) ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(src: &str) -> f64 {
        let mut i = Interpreter::new();
        match i.run(src).unwrap().unwrap() {
            Outcome::Value(v) | Outcome::Assigned(_, v) => v,
        }
    }

    #[test]
    fn precedence() {
        assert_eq!(eval("1 + 2 * 3"), 7.0);
        assert_eq!(eval("(1 + 2) * 3"), 9.0);
        assert_eq!(eval("10 - 4 - 3"), 3.0);
        assert_eq!(eval("2 ^ 3 ^ 2"), 512.0);
        assert_eq!(eval("-2 ^ 2"), -4.0);
    }

    #[test]
    fn variables_and_functions() {
        let mut i = Interpreter::new();
        i.run("let r = 3").unwrap();
        match i.run("pi * r ^ 2").unwrap().unwrap() {
            Outcome::Value(v) => assert!((v - 28.2743).abs() < 1e-3),
            _ => panic!(),
        }
        assert_eq!(eval("sqrt(16) + max(2, 5)"), 9.0);
    }

    #[test]
    fn errors() {
        let mut i = Interpreter::new();
        assert!(i.run("1 / 0").is_err());
        assert!(i.run("x + 1").is_err());
        assert!(i.run("(1 + 2").is_err());
        assert!(i.run("1 $ 2").is_err());
    }
}
