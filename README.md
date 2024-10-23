![Build status](https://github.com/aripot007/kompilattheur/actions/workflows/rust.yml/badge.svg)

## Table of Contents

1. [Introduction](#introduction)
1. [Team](#team)
1. [Features](#features)
1. [Installation](#installation)
1. [Usage](#usage)
1. [Examples](#examples)
1. [Testing](#testing)
1. [Acknowledgements](#acknowledgements)

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
- **Error Reporting:** Detailed syntax and semantic error messages

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

## Usage

### Basic Command

To compile a MiniPython source file:

```bash
$ kompillatheur [source-file]
```

### Command-line Options

None yet

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
