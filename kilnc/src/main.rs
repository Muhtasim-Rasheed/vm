mod codegen;
mod ir;
mod parsing;
mod semantic_checker;

struct TeeWriter<W1: std::io::Write, W2: std::io::Write> {
    a: W1,
    b: W2,
}

impl<W1: std::io::Write, W2: std::io::Write> std::io::Write for TeeWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.a.write(buf)?;
        self.b.write_all(&buf[..n])?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.a.flush()?;
        self.b.flush()
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("Backtrace:\n{}", backtrace);
        eprintln!("Error: {}", info);
    }));

    let mut args = std::env::args();
    let _ = args.next();
    let file_path_str = args.next().expect("No file path provided");
    let file_path = std::path::Path::new(&file_path_str);
    if !file_path.exists() {
        panic!("File does not exist: {}", file_path_str);
    }
    let out_path_str = args.next().expect("No output path provided");
    let out_path = std::path::Path::new(&out_path_str);
    let source = std::fs::read_to_string(file_path).expect("Failed to read file");
    let mut lexer = parsing::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap_or_else(|e| panic!("{}", e));
    let mut parser = parsing::parser::Parser::new(tokens, &source);
    let mut ast = parser.parse().unwrap_or_else(|e| panic!("{}", e));
    let mut checker = semantic_checker::SemanticChecker::new(&mut ast, &source);
    checker.check().unwrap_or_else(|e| panic!("{}", e));
    let ir_module = ir::lower::IrModuleBuilder::new(&ast).lower();
    let out = TeeWriter {
        a: std::io::stdout(),
        b: std::fs::File::create(out_path).expect("Failed to create output file"),
    };
    let codegen = codegen::CodeGenerator::new(ir_module, Box::new(out));
    codegen.lower().unwrap_or_else(|e| panic!("{}", e));
}
