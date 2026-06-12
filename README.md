# Vyākṛti — Sanskrit-Oriented Programming Language & Web IDE

> **A working MVP for a Sanskrit-oriented programming language with a compiler, bytecode VM, CLI, and web-based IDE.**

Vyākṛti (Sanskrit: व्याकृतिः, "structured form") is an experimental programming language that uses Devanagari script and Sanskrit-derived keywords as its syntax. This repository contains the entire project: the language compiler/VM library, a CLI tool, and a React-based web IDE with Monaco editor integration.

**Current status:** Working MVP. The language compiles and runs. The web IDE has a complete UI. Frontend and backend run separately. No auth, no database, no AI layer yet.

---

## Table of Contents

- [What's Included](#whats-included)
- [Architecture](#architecture)
- [Repository Structure](#repository-structure)
- [Tech Stack](#tech-stack)
- [Quick Start](#quick-start)
- [What Works](#what-works)
- [What's Not Done Yet](#whats-not-done-yet)
- [Screenshots](#screenshots)
- [License](#license)

---

## What's Included

- **Language core** — Lexer, parser, semantic type checker, bytecode compiler, stack-based VM
- **CLI tool** — `vy compile`, `vy run`, `vy repl` commands
- **Web IDE** — React + Monaco Editor with syntax highlighting, autocomplete, hover tooltips, diagnostics, file management, command palette, WebSocket REPL
- **Self-hosting corpus** — The language's own lexer/parser written in Vyākṛti as test fixtures
- **123 tests** — Covering lexer, parser, semantic analysis, VM, optimizer, borrow checker, exhaustiveness, and end-to-end pipeline

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Vyākṛti Project                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              vyakrti-language/ (Core)                     │   │
│  │                                                          │   │
│  │  Source → Lexer → Parser → Type Checker → Compiler → VM  │   │
│  │                                                          │   │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐         │   │
│  │  │ Lexer  │→ │ Parser │→ │Semantic│→ │Compiler│         │   │
│  │  │        │  │        │  │(kāraka)│  │        │         │   │
│  │  └────────┘  └────────┘  └────────┘  └────────┘         │   │
│  │                                              │           │   │
│  │                                              ▼           │   │
│  │                                         ┌────────┐       │   │
│  │                                         │   VM   │       │   │
│  │                                         │(stack) │       │   │
│  │                                         └────────┘       │   │
│  │                                                          │   │
│  │  CLI: vy compile | vy run | vy repl                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              vyakrti-ide/ (Web IDE)                       │   │
│  │                                                          │   │
│  │  ┌─────────────────┐   ┌─────────────────────────────┐  │   │
│  │  │   frontend/      │   │   backend/                   │  │   │
│  │  │   React + TS     │──▶│   Rust + axum                │  │   │
│  │  │   Monaco Editor  │   │   REST + WebSocket           │  │   │
│  │  │   Zustand        │   │   Compile endpoint           │  │   │
│  │  │   Tailwind CSS   │   │   LSP endpoints              │  │   │
│  │  └─────────────────┘   │   WebSocket REPL             │  │   │
│  │                         └─────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Compiler Pipeline

```
Source Code (Devanagari)
    │
    ▼
Lexer ──→ Token stream (38+ token types, Unicode-aware)
    │
    ▼
Parser ──→ AST (recursive-descent, expressions + statements)
    │
    ▼
Semantic Type Checker ──→ Type-checked AST (kāraka-driven scoping)
    │
    ▼
Bytecode Compiler ──→ Binary bytecode (34 opcodes)
    │
    ▼
Virtual Machine ──→ Execution (stack-based, with builtins)
```

---

## Repository Structure

```
Vyakrti/
├── README.md                   ← You are here
├── .gitignore
│
├── vyakrti-language/           ← Core language library + CLI
│   ├── Cargo.toml              ← Rust package: "vyakriti" v2026.1.0
│   ├── src/
│   │   ├── lib.rs              ← Library root (re-exports all modules)
│   │   ├── main.rs             ← Example pipeline runner + tests
│   │   ├── bin/vy.rs           ← CLI binary (compile / run / repl)
│   │   ├── lexer.rs            ← Tokenizer (Devanagari + Latin)
│   │   ├── parser.rs           ← Recursive-descent parser → AST
│   │   ├── ast.rs              ← AST node types
│   │   ├── semantic.rs         ← Kāraka-driven type checker
│   │   ├── compiler.rs         ← Bytecode compiler
│   │   ├── vm.rs               ← Stack-based virtual machine
│   │   ├── borrow_checker.rs   ← Ownership verification
│   │   ├── optimizer.rs        ← Constant folding
│   │   ├── exhaustiveness.rs   ← Match coverage analysis
│   │   ├── macro_expander.rs   ← Macro template expansion
│   │   ├── monomorphizer.rs    ← Generics monomorphization
│   │   ├── derive_processor.rs ← Attribute auto-derivation
│   │   ├── disassembler.rs     ← Bytecode disassembler
│   │   ├── jit_memory.rs       ← JIT memory (stub)
│   │   └── jit_compiler.rs     ← JIT compilation (stub)
│   ├── std/                    ← Vyākṛti standard library
│   ├── selfhost/               ← Self-hosting test corpus
│   └── tests/
│       ├── tantram.vya         ← Integration test fixture
│       └── ai_harness.rs       ← 82 AI-generated integration tests
│
├── vyakrti-ide/                ← Web IDE (frontend + backend server)
│   ├── start-ide.bat           ← Windows launcher
│   ├── start-ide.ps1           ← PowerShell launcher
│   ├── frontend/               ← React web application
│   │   ├── package.json        ← Node deps: React, Monaco, Zustand, Vite
│   │   ├── src/
│   │   │   ├── App.tsx         ← Root component (IDE layout)
│   │   │   ├── store/ideStore.ts ← Zustand state management
│   │   │   ├── utils/vyakritiLanguage.ts ← Monaco language definition
│   │   │   └── components/
│   │   │       ├── Editor/     ← CodeEditor, EditorTabs
│   │   │       ├── Sidebar/    ← ProjectExplorer
│   │   │       ├── Toolbar/    ← TopBar
│   │   │       ├── Panels/     ← DiagnosticsConsole, StatusBar,
│   │   │       │                 SettingsPanel, ToolPanels, ErrorBoundary
│   │   │       └── Modals/     ← CommandPalette, WorkspaceModals
│   │   └── index.html
│   └── backend/                ← Rust backend server
│       ├── Cargo.toml          ← axum + tokio + vyakriti crate
│       └── src/
│           ├── main.rs         ← axum server (REST + WebSocket)
│           ├── compiler.rs     ← Compile endpoint
│           ├── lsp.rs          ← LSP-like endpoints
│           ├── ws.rs           ← WebSocket REPL handler
│           ├── workspace.rs    ← File system operations
│           └── payload.rs      ← Request/response types
│
└── screenshots/                ← IDE screenshots (see Screenshots section)
```

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language Core | Rust |
| CLI | Rust binary |
| IDE Backend | Rust (axum, tokio, serde) |
| IDE Frontend | React 18, TypeScript, Vite |
| Code Editor | Monaco Editor |
| State Management | Zustand |
| Styling | Tailwind CSS |
| Testing | Rust built-in test framework |
| Version Control | Git + GitHub |

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+) — for the web IDE frontend

### 1. Clone

```bash
git clone https://github.com/yourusername/Vyakrti.git
cd Vyakrti
```

### 2. Run the Language CLI

```bash
cd vyakrti-language

# Compile and run a Vyākṛti source file
cargo run --bin vy -- compile selfhost/golden_smoke.vya

# Start the interactive REPL
cargo run --bin vy -- repl

# Run tests
cargo test
```

### 3. Run the Web IDE

Terminal 1 — start the backend:
```bash
cd vyakrti-ide/backend
cargo run
# Server listens on http://127.0.0.1:8080
```

Terminal 2 — start the frontend:
```bash
cd vyakrti-ide/frontend
npm install
npm run dev
# Open http://localhost:5173
```

---

## What Works

### Language Core
- **Lexer** — Tokenizes Devanagari source; recognizes 38+ token types including Devanagari digits, danda (।) terminators, mixed-script identifiers
- **Parser** — Recursive-descent parser producing AST for variables, functions, conditionals, loops, enums, structs, traits, pattern matching
- **Semantic type checker** — Scope-aware symbol table with kāraka-driven role extraction from variable names
- **Bytecode compiler** — Serializes AST to binary bytecode with 34 opcodes
- **Virtual machine** — Stack-based execution with builtins (print, length, type, concat, contains, parse-int, to-string)
- **CLI** — `vy compile`, `vy run`, `vy repl` commands
- **Self-hosting corpus** — Language's own lexer/parser written in Vyākṛti as test fixtures

### Web IDE
- **Monaco code editor** with Vyākṛti syntax highlighting (Devanagari keywords, danda delimiters)
- **Hover tooltips** showing Sanskrit keyword meanings
- **Autocomplete** with keyword and snippet support
- **Compile & Run** — sends source to backend, displays tokens, AST, bytecode, diagnostics, output
- **WebSocket REPL** — interactive evaluation
- **File management** — create, rename, delete, save files
- **Project explorer** sidebar
- **Settings panel** with theme toggle
- **Command palette** (Ctrl+K)
- **Error boundaries** — UI doesn't crash on component errors

### Testing
- **123 tests** covering all major components
- End-to-end pipeline tests
- Self-hosting corpus tests (language parses its own source)

---

## What's Not Done Yet

Honest list of current limitations:

| Feature | Status | Notes |
|---------|--------|-------|
| AI/LLM integration | ❌ Not implemented | The `explain` endpoint uses hardcoded pattern matching, not actual Ollama/OpenAI calls |
| Database persistence | ❌ Not implemented | No user accounts, no project saving |
| Authentication | ❌ Not implemented | No login system |
| Module system | ❌ Not implemented | `आयात` (import) is parsed but not resolved |
| Multi-file projects | ❌ Not implemented | No file import/linking |
| Float arithmetic | ⚠️ Bug | Float addition causes stack underflow in VM (loading float literals works) |
| JIT compilation | ❌ Stub | `jit_compiler.rs` and `jit_memory.rs` are empty |
| FFI | ⚠️ Unsafe | Uses `unsafe` + `mem::forget`, leaks memory |
| Deployment | ❌ Not set up | Frontend and backend run separately, no Docker/CI |
| Frontend tests | ❌ None | No Jest/Vitest tests for React components |

---

## Screenshots

See the [`screenshots/`](screenshots/) directory for IDE screenshots.

To capture your own:
1. Start the IDE (see Quick Start)
2. Open a `.vya` file in the editor
3. Save screenshots to `screenshots/`

Suggested captures:
- `ide-editor.png` — Editor with syntax highlighting
- `ide-compile-output.png` — Compile output with AST/bytecode
- `ide-repl.png` — WebSocket REPL session
- `ide-settings.png` — Settings panel

---

## Language Example

```vyakriti
// Variable declaration
मान मूल्यम् : अङ्क = ६० * ६० ।

// Function declaration
कार्य योगः(क : अङ्क, ख : अङ्क) -> अङ्क {
    प्रतिफल क + ख ।
}

// Conditional
यदि (सत) तर्हि {
    मुद्रण("नमो व्याकृतिः") ।
}
```

### Keywords

| Vyākṛti | English | Vyākṛti | English |
|----------|---------|----------|---------|
| मान | var | कार्य | function |
| प्रतिफल | return | मुद्रण | print (builtin) |
| यदि | if | तर्हि | then |
| अन्यथा | else | यावत् | while |
| तावत् | do | सत / असत | true / false |
| च / वा | and / or | समीक्षा | match |
| वस्तु_विन्यासः | struct | रूपभेदः | enum |
| गुणधर्म | trait | अनुष्ठान | impl |
| उदात्त / अनुदात्त / स्वरित | public / private / protected | | |

---

## License

This project is an experimental prototype under active development.
