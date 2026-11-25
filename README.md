# calc_lang

A tiny programming language for math, written in Rust.

`calc_lang` is a small interpreter built to learn how programming languages work. It reads math expressions, understands variables and functions, and calculates the result. You can type lines one by one in an interactive prompt, or run a whole `.calc` file.

```
> 1 + 2 * 3
7
> let r = 5
r = 5
> pi * r ^ 2
78.53981633974483
> sqrt(3^2 + 4^2)
5
```

## Features

- **Numbers:** integers and decimals (`42`, `3.14`)
- **Operators:** `+` `-` `*` `/` `%` `^` (power)
- **Order of operations:** `1 + 2 * 3` is `7`, and `2 ^ 3 ^ 2` is `512` (power is right-associative)
- **Parentheses:** `(1 + 2) * 3`
- **Negative numbers:** `-5`, `-(2 + 3)`
- **Variables:** `let x = 10`, then use `x` in later lines
- **Constants:** `pi`, `e`
- **Built-in functions:** `sqrt`, `abs`, `sin`, `cos`, `tan`, `ln`, `floor`, `ceil`, `round`, `min`, `max`
- **Comments:** everything after `#` on a line is ignored
- **Clear errors:** `division by zero`, `unknown variable 'x'`, `expected RParen, found Eof`, and so on

## Getting started

You need [Rust](https://www.rust-lang.org/tools/install) installed (it includes `cargo`).

```bash
git clone https://github.com/<your-username>/calc_lang.git
cd calc_lang
cargo build --release
```

### Interactive prompt (REPL)

```bash
cargo run
```

Type an expression and press Enter. Type `exit` (or press Ctrl+D) to quit.

### Run a file

```bash
cargo run -- examples/demo.calc
```

Example program (`examples/demo.calc`):

```
# shopping total with 21% tax
let price = 19.99
let qty = 3
let subtotal = price * qty
let tax = subtotal * 0.21
subtotal + tax
```

Output:

```
price = 19.99
qty = 3
subtotal = 59.97
tax = 12.5937
72.5637
```

### Run the tests

```bash
cargo test
```

## How it works

Every interpreter follows the same pipeline, and `src/main.rs` is split into these sections in the same order:

```
source text ──► Lexer ──► tokens ──► Parser ──► AST ──► Evaluator ──► result
```

1. **Lexer** (`tokenize`): turns the text `let x = 1 + 2` into tokens: `Let`, `Ident("x")`, `Equals`, `Number(1)`, `Plus`, `Number(2)`.
2. **Parser** (`Parser`): a recursive descent parser that turns tokens into an Abstract Syntax Tree (AST). Each precedence level has its own function, so `1 + 2 * 3` becomes:
   ```
        Add
       /   \
      1    Mul
          /   \
         2     3
   ```
3. **Evaluator** (`Interpreter`): walks the tree and computes the value. Variables live in a `HashMap`.

### Grammar

```
statement := "let" IDENT "=" expr | expr
expr      := term (("+" | "-") term)*
term      := unary (("*" | "/" | "%") unary)*
unary     := "-" unary | power
power     := primary ("^" unary)?
primary   := NUMBER | IDENT | IDENT "(" args ")" | "(" expr ")"
args      := expr ("," expr)*
```

## Roadmap

- [x] Arithmetic with correct precedence
- [x] Variables and constants
- [x] Built-in math functions
- [x] REPL and file execution
- [ ] Comparisons (`<`, `>`, `==`, `!=`)
- [ ] `if` / `else`
- [ ] User-defined functions (`fn square(x) = x * x`)
- [ ] Loops
- [ ] Strings and printing
- [ ] Better error messages with line and column numbers

## Project structure

```
calc_lang/
├── Cargo.toml
├── README.md
├── examples/
│   └── demo.calc
└── src/
    └── main.rs      # lexer, parser, evaluator, REPL and tests
```

## License

MIT
