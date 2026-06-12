#![allow(clippy::type_complexity)]
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod semantic;
pub mod macro_expander;
pub mod derive_processor;
pub mod monomorphizer;
pub mod optimizer;
pub mod borrow_checker;
pub mod exhaustiveness;
pub mod compiler;
pub mod jit_memory;
pub mod jit_compiler;
pub mod vm;
pub mod disassembler;
pub mod import_resolver;
