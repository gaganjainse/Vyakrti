# 🕉️ Vyākṛti

> **A Sanskrit-oriented programming language with a complete compiler pipeline**
> (lexer → parser → kāraka-driven type checker → bytecode + VM) and a browser IDE.
> An experimental MVP — not a production deployment.

![Rust](https://img.shields.io/badge/Rust-orange?logo=rust) ![License](https://img.shields.io/badge/License-GPL--3.0--or--later-blue) ![Tests](https://img.shields.io/badge/Tests-127-success) ![CI](https://github.com/gaganjainse/Vyakrti/actions/workflows/rust.yml/badge.svg)

- **License:** GPL-3.0-or-later
- **Owner:** Gagan Jain ([@gaganjainse](https://github.com/gaganjainse))
- **Stack:** Rust (Axum) · React + Monaco (web IDE) · TypeScript

---

## Why this repo exists

An exploration of programming-language design, developer tooling, and web-IDE
architecture — integrating Sanskrit linguistic principles into a full compiler
pipeline.

## Features

- **Custom language** — Sanskrit-oriented syntax from scratch
- **Complete compiler pipeline** — recursive-descent lexer, parser, kāraka-driven semantic type checker, bytecode compiler (34 opcodes), stack VM
- **Browser IDE** — React + Monaco with syntax highlighting, autocomplete, diagnostics
- **Rust backend** — Axum: compile, REPL, LSP, file management via REST + WebSocket
- **CLI** — `vy compile`, `vy run`, `vy repl`
- **Testing** — 127 tests incl. a self-hosting corpus

## Quick start

```bash
cargo build --release
cargo test             # 127 tests
vy run examples/hello.vya
```

## Repository layout

```text
Vyakrti/
├── vyakrti-language/       # compiler core (lexer..VM)
├── vyakrti-ide/backend/    # axum API for the web IDE
└── vyakrti-ide/            # React + Monaco frontend
```

## Status

CI green. Security: [SECURITY.md](SECURITY.md).

## Documentation index

- **Compiled reading:** [shesh-docs](https://github.com/gaganjainse/shesh-docs)

## License

GPL-3.0-or-later — see [LICENSE](LICENSE).
