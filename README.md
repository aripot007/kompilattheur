![Build status](https://github.com/aripot007/kompilattheur/actions/workflows/rust.yml/badge.svg)

## Table of Contents

1. [Introduction](#introduction)
1. [Team](#team)
1. [Features](#features)
1. [Installation](#installation)
1. [Usage](#usage)
1. [Examples](#examples)
1. [Testing](#testing)
1. [Contributing](#contributing)

---

## Introduction

The **Kompilattheur Compiler** is "a MiniPython compiler designed to compile MiniPython. It translates MiniPython source code into intermediate representation.

### Team

- Baptiste JULLIEN
- Luca MANDRELLI
- Romain PONSON--LISSALDE
- Aristide URLI

### Why use Kompilattheur?

- Rust.
- Why not ?

## Features

- **Cross-platform:** Runs on linux
- **Optimization:** None yet
- **Error Reporting:** Detailed syntax and semantic error message
- **Analysis table generation:** Generate analysis tables with the `print-analysis-table` and `generate-analysis-table` commands
- **Runtime optional grammar parsing:** You can use an alternative grammar at runtime with the `--alternative-grammar` option
- **Shell autocompletion:** Generate autocompletion scripts for your shell

## Installation

### Prerequisites

- **Rust:** Ensure you have a [Rust](https://www.rust-lang.org/tools/install) compiler installed.

### Installing the Compiler

Clone the repository and install any required dependencies:

```bash
$ git clone https://gibson.telecomnancy.univ-lorraine.fr/projets/2425/compil/kompillatheur.git
$ cd kompillatheur
$ cargo install --path .
```

To compile the compiler without installing it, use :

For debug build :

```bash
cargo build
```

For release build :
```bash
cargo build --profile release
```

### Installing auto-completion

The compiler can generate auto-completion scripts for most shells :

For bash
```bash
kompilattheur generate-autocompletion bash  > /usr/share/bash-completion/completions/kompilattheur.bash
```

```bash
kompilattheur generate-autocompletion zsh  > /usr/local/share/zsh/site-functions/_kompilattheur
```

Currently supported shells are `bash`, `elvish`, `fish`, `powershell` and `zsh`.

To install the auto-completion script, refer to your shell's manual.

## Usage

### Basic Command

To compile a MiniPython source file:

```bash
$ kompillatheur [source-file]
```

### Command-line Options

Use `kompillatheur help` to see all command line options

### Analysis tables

#### Generating an analysis table

You can generate an analysis table for a grammar with the `generate-analysis-table` subcommand.
The grammar must be in a file that follows the [grammophone](https://mdaines.github.io/grammophone/#/) syntax.

```bash
kompilattheur generate-analysis-table grammaire.txt -o src/analysis_table/generated_table.rs
```

Optionnaly, you can make the generator add comments to the file for easier debugging wiht the `--with-comments` option.

#### Printing an analysis table

You can print the compiler's analysis table or a generated analysis table with the `print-analysis-table` subcommand.
Printing is supported in different formats :
- `plaintext` : Readable plain text
- `markdown` : Markdown table format, most readable when rendered
- `rust` : Rust source code, equivalent to using `generate-analysis-table`

```bash
# Print the built-in analysis table in markdown format to the analysis table.md file
kompilattheur print-analysis-table --format markdown -o analysis_table.md

# Generate and print an analysis table for the grammar_ex.txt grammar
kompilattheur print-analysis-table -g grammaire_ex.txt
```

#### Using another grammar at runtime

You can generate an analysis table to use for compilation from another grammar at runtime with the `--alternative-grammar` option :

```bash
kompilattheur -g grammaire_ex.txt test_programs/arithmetic.smolpp
```

### Examples

#### Compiling a simple program:

```bash
$ kompillatheur example.smolpp
```



## Examples

For more examples, refer to the `test_programs/` directory.

## Testing

To run tests, use
```bash
$ cargo test
```

### Reporting Issues

If you encounter any issues or have feature requests, please open an issue on the GitHub repository.

## Contributing

### Setting up Git hooks

To ensure consistent code formatting, the project includes git hooks. After cloning, run:

```bash
./setup-hooks.sh
```

This will set up a pre-commit hook that runs `cargo fmt --check` before each commit to ensure proper code formatting.