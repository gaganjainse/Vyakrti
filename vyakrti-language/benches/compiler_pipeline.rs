//! Compiler hot-path benchmarks: lexer + parser throughput.
//!
//! The lex/parse cycle is what runs on every keystroke in the IDE, so its
//! throughput matters. Benches use a representative program of each size;
//! CI runs `cargo bench -- --test` (single iteration, compile+run check)
//! so timing noise never fails CI — full timing is for local runs.

use criterion::{criterion_group, criterion_main, Criterion};

use vyakriti::lexer::Lexer;
use vyakriti::parser::Parser;

const SMALL_PROGRAM: &str = r#"
वचन नमस्ते {
    लिखो("Hello, World!");
}
"#;

const MEDIUM_PROGRAM: &str = r#"
वचन गणना_करो(संख्या: पूर्णांक) {
    यदि (संख्या > 10) {
        लिखो("बड़ी संख्या");
    } अन्यथा {
        लिखो("छोटी संख्या");
    }
    लौटाओ(संख्या * 2);
}

मुख्य() {
    गणना_करो(5);
    गणना_करो(42);
    गणना_करो(99);
}
"#;

fn lex(program: &str) -> usize {
    let mut lexer = Lexer::new(program);
    lexer.tokenize().len()
}

fn lex_and_parse(program: &str) -> usize {
    let mut lexer = Lexer::new(program);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_program().map(|n| n.len()).unwrap_or(0)
}

pub fn bench_lexer(c: &mut Criterion) {
    c.bench_function("lex small program", |b| b.iter(|| lex(SMALL_PROGRAM)));
    c.bench_function("lex medium program", |b| b.iter(|| lex(MEDIUM_PROGRAM)));
}

pub fn bench_parser(c: &mut Criterion) {
    c.bench_function("lex+parse small program", |b| b.iter(|| lex_and_parse(SMALL_PROGRAM)));
    c.bench_function("lex+parse medium program", |b| b.iter(|| lex_and_parse(MEDIUM_PROGRAM)));
}

criterion_group!(benches, bench_lexer, bench_parser);
criterion_main!(benches);
