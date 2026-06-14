# Vyākṛti: A Sanskrit-oriented Programming Language and Web IDE

Vyākṛti is an ambitious engineering project exploring programming language design, developer tooling, web IDE architecture, and AI-assisted workflows. It is an experimental Sanskrit-oriented programming language with a complete compiler pipeline and a browser-based Integrated Development Environment (IDE).

## Overview

Vyākṛti aims to provide a unique programming experience by integrating Sanskrit linguistic principles into its design. The project encompasses a full compiler pipeline, a command-line interface (CLI), and an interactive web-based IDE for writing, compiling, and running Vyākṛti code.

## Features

*   **Custom Programming Language:** Designed and implemented a Sanskrit-oriented programming language from scratch.
*   **Complete Compiler Pipeline:** Includes a recursive-descent lexer, parser, kāraka-driven semantic type checker, bytecode compiler (34 opcodes), and a stack-based virtual machine (VM).
*   **Browser-Based IDE:** An interactive web IDE built with React and Monaco Editor, offering syntax highlighting, autocomplete, and diagnostics for Vyākṛti code.
*   **Rust-based Backend:** An Axum (Rust) backend provides compile, REPL (Read-Eval-Print Loop), LSP (Language Server Protocol), and file-management endpoints via REST and WebSocket.
*   **CLI Tool:** A command-line interface with `vy compile`, `vy run`, and `vy repl` commands for local development.
*   **Comprehensive Testing:** 123 tests covering the full compiler pipeline, including a self-hosting corpus where the language parses its own source code.

## Tech Stack

*   **Language Core & IDE Backend:** Rust (Axum framework)
*   **Web IDE Frontend:** React 18, TypeScript, Vite, Monaco Editor, Zustand, Tailwind CSS
*   **Testing:** Custom test suite, self-hosting corpus

## Architecture Summary

The Vyākṛti system is composed of several interconnected components:

1.  **Compiler Frontend (Rust):** Handles lexical analysis (lexer), parsing (recursive-descent parser), and semantic analysis (kāraka-driven type checker) to transform source code into an Abstract Syntax Tree (AST).
2.  **Compiler Backend (Rust):** Converts the AST into bytecode, which is then executed by a custom stack-based Virtual Machine (VM).
3.  **Web IDE (React, TypeScript):** Provides the user interface for writing and interacting with Vyākṛti code. It communicates with the backend via REST and WebSockets for compilation, execution, and language services.
4.  **Backend Server (Axum, Rust):** Exposes API endpoints for the web IDE and CLI, handling compilation requests, REPL interactions, LSP services, and file management.
5.  **CLI (Rust):** Offers command-line access to the compiler and VM functionalities.

```mermaid
graph TD
    A[Vyākṛti Source Code] --> B{Lexer}
    B --> C{Parser}
    C --> D{Semantic Type Checker}
    D --> E{Bytecode Compiler}
    E --> F[Stack-based VM]
    F --> G[Execution Result]
    H[Web IDE (React)] -- REST/WebSocket --> I[Backend Server (Axum Rust)]
    J[CLI] -- API Calls --> I
    I -- Compiler Pipeline --> A
```

## Getting Started

To get started with Vyākṛti, you will need Rust and Node.js installed. Clone the repository and follow the instructions in the `CONTRIBUTING.md` file for setting up the development environment.

```bash
git clone https://github.com/gaganjainse/Vyakrti.git
cd Vyakrti
# Follow instructions in CONTRIBUTING.md for specific build steps
```

## Screenshots / Demo Notes

*(Placeholder for screenshots or GIF of the Web IDE in action, showcasing syntax highlighting, compilation, and REPL interaction.)*

## Limitations / Future Work

*   **AI/LLM Integration:** Currently a stub; future plans include integrating AI/LLM capabilities for code generation, auto-completion, and intelligent error correction.
*   **Module System:** A robust module system is planned to enable better code organization and reusability.
*   **JIT Compilation:** Just-In-Time (JIT) compilation is a future consideration for performance optimization.
*   **Authentication & Database:** The current IDE does not include user authentication or a database. These are planned for future iterations to support multi-user environments and project persistence.
*   **No Production Claims:** This project is an experimental MVP and is not deployed in a production environment with real users or live traffic.

## Cross-links

*   **GitHub Profile:** [https://github.com/gaganjainse](https://github.com/gaganjainse)
*   **LinkedIn Profile:** [https://linkedin.com/in/gagan-jain-a88aab345](https://linkedin.com/in/gagan-jain-a88aab345)
*   **Portfolio:** [https://gagan-jain-portfolio.vercel.app](https://gagan-jain-portfolio.vercel.app)

---

*Last updated: June 14, 2026*
