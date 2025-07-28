# TreeScan

*This is a proof of concept and work in progress*

Multi-language AST (Abstract Syntax Tree) parser and code quality analyzer built in Rust. TreeScan provides both a command-line interface for interactive use and a C-compatible library for integration into other projects.

## Features

- **Multi-language AST Parsing**: Parse source code into readable AST representations
- **Code Quality Analysis**: Analyze code and generate quality metrics and scores
- **C FFI library interface**: for cross-language integration
- **Language Support**: Supports Rust, JavaScript, TypeScript, Java, C/C++, Zig, and Go

## Supported Languages

### AST Parsing
- Rust (`.rs`)
- Java (`.java`)
- Zig (`.zig`)
- C/C++ (`.c`, `.h`, `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`)
- JavaScript (`.js`, `.jsx`)
- TypeScript (`.ts`, `.tsx`)

### Code Analysis
- Rust (`.rs`)
- Go (`.go`)
- JavaScript (`.js`, `.jsx`)

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd treescan

# Build the project
cargo build --release
```

## Usage

### Command Line Interface

#### Parse a file to view its AST:
```bash
# Auto-detect language from file extension
treescan parse src/main.rs
treescan parse script.js
treescan parse hello.c
```

#### Analyze code quality:
```bash
# Analyze code and get quality metrics
treescan analyze src/main.rs
treescan analyze main.go
treescan analyze script.js
```

### Library Usage

TreeScan can be used as a library through its C FFI interface:

## Example Output

### AST Parsing
```
Parsing Rust file: src/main.rs
----------------------------------------
(source_file
  (function_item
    (visibility_modifier "pub")
    (identifier "main")
    (parameters)
    (block
      (expression_statement
        (macro_invocation
          (identifier "println!")
          (token_tree "\"Hello, world!\"")))))
```

### Code Analysis
```
Analyzing Rust file: src/main.rs
----------------------------------------
{
  "complexity": 1,
  "quality_score": 85,
  "metrics": {
    "lines_of_code": 10,
    "cyclomatic_complexity": 1,
    "maintainability_index": 85
  },
  "issues": []
}
```

## Acknowledgments

This project is built on top of [Tree-sitter](https://tree-sitter.github.io/tree-sitter/),
a parser generator tool and incremental parsing library created by GitHub.

- Tree-sitter core library
- Tree-sitter language grammars for Rust, JavaScript, Java, C/C++, etc.
