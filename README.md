# 🇵🇭 Tol Programming Language (`tol-lang`)

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-brightgreen.svg)](Cargo.toml)

**Tol** (short for *Tagalog-Oriented Language*) is a modern, indentation-based programming language featuring native Tagalog keywords, a bytecode-compiled stack-based virtual machine runtime, and standard library modules written in Rust.

Designed to make programming intuitive and expressive in Tagalog, **Tol** combines accessible syntax with compiler diagnostics powered by `miette`.

---

## 🚀 Key Features

* **Native Tagalog Syntax:** Program using natural Tagalog keywords like `ang`, `paraan`, `kung`, `habang`, and `ibalik`.
* **Clean Indentation Scoping:** Python-style block scoping using clean whitespace indents.
* **Bytecode VM Execution:** High-performance bytecode generation and stack-based virtual machine runtime.
* **Rich Diagnostic Reports:** User-friendly error messages with span highlighting powered by `miette`.
* **Standard Library Support:** Built-in modules for I/O (`io`), mathematical operations (`math`), uri (types) (`uri`), string conversions (`Teksto`), list manipulation (`Lista`), and ranges (`Sakop`).
* **Strict Typing**: The user cannot directly do possibly dangerous operations such as adding integers (`numero`) and floats (`lutang`) together

---

## 📚 Keyword Reference

| Keyword | Equivalent in English / Python | Description |
| :--- | :--- | :--- |
| `ang` | `let` / `var` (in javascript) | Variable declaration |
| `paraan` | `def` | Function definition |
| `kung` | `if` | Conditional branch |
| `kundi` | `elif` | Alternative conditional branch |
| `kungwala` | `else` | Fallback conditional branch |
| `habang` | `while` | While loop construct |
| `bawat ... sa ...` | `for ... in ...` | For-each loop construct |
| `biyakin` | `break` | Terminate loop execution |
| `ituloy` | `continue` | Proceed to next loop iteration |
| `ibalik` | `return` | Return value from function |
| `klase` | `class` | Class or data structure definition |
| `kunin` | `import` | Import module or stdlib component |
| `totoo` | `true` | Boolean true value |
| `mali` | `false` | Boolean false value |
| `at` | `and` | Logical AND operator |
| `o` | `or` | Logical OR operator |
| `di` | `not` | Logical NOT operator |

---

## 📥 Installation

### Quick Install (Linux / macOS)

Install [Rust](https://www.rust-lang.org/tools/install) first

Clone the repository and execute the installation script:

```bash
git clone https://github.com/wrkean/tol-lang.git
cd tol-lang
chmod +x install.sh
./install.sh
```

This installs the `tol` binary into `$HOME/.local/bin` and copies the standard library to `$HOME/.local/share/tol`. It also exports the `TOL_STDLIB` environment variable in your profile.

### Windows Setup

A Windows setup script is available in the [Releases](https://github.com/wrkean/tol-lang/releases) page.

### Manual Build

Install [Rust](https://www.rust-lang.org/tools/install) first

```bash
cargo build --release
```

Ensure the environment variable `TOL_STDLIB` points to the `stdlib` directory:

```bash
export TOL_STDLIB="$(pwd)/stdlib"
```

---

## 🛠️ Usage

Use the `takbo` command (*run*) to compile and execute a `.tol` script:

```bash
tol takbo path/to/script.tol
```

Or using Cargo during development:

```bash
TOL_STDLIB=stdlib cargo run -- takbo examples/kumusta.tol
```

---

## 💻 Code Examples

### 1. Hello World (`kumusta.tol`)

```tol
io.isulatln("Kumusta mundo!")
```

### 2. Variables & Math Constants

```tol
ang pangalan = "Tol"
ang PI_VALUE = math.PI

io.isulatln("Mabuhay {}, PI = {}", pangalan, PI_VALUE)
```

### 3. Functions & Control Flow

```tol
paraan batiin(pangalan):
    kung pangalan == "Juan":
        io.isulatln("Maligayang pagdating, Juan!")
    kungwala:
        io.isulatln("Kumusta, {}!", pangalan)

batiin("Juan")
```

### 4. Loops & Iteration

```tol
# Iterating over range
bawat numero sa 0..=5:
    io.isulatln("Bilang: {}", numero)

# While loop
ang counter = 0
habang counter < 3:
    io.isulatln("Counter: {}", counter)
    counter = counter + 1
```

---

## 🏗️ Architecture

The **Tol** compiler and runtime consist of five main stages:

```
Source Code (.tol)
       │
       ▼
   [ Lexer ] ─────► Converts source text into Token streams (handling indents & dedents)
       │
       ▼
   [ Parser ] ────► Builds Abstract Syntax Tree (AST)
       │
       ▼
   [ Analyzer ] ──► Performs name resolution
       │
       ▼
   [ Codegen ] ───► Emits bytecode instructions
       │
       ▼
   [ VM ] ────────► Executes bytecode on the stack-based virtual machine
```

---

## 📜 License

This project is licensed under the [MIT License](LICENSE) - see the LICENSE file for details.

Developed with ❤️ by **Keanne Barraca**.
