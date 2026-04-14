mod ir;
mod parsing;
mod semantic_checker;

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let file_path_str = args.next().expect("No file path provided");
    let file_path = std::path::Path::new(&file_path_str);
    if !file_path.exists() {
        panic!("File does not exist: {}", file_path_str);
    }
    let out_path_str = args.next().expect("No output path provided");
    let _out_path = std::path::Path::new(&out_path_str);
    let source = std::fs::read_to_string(file_path).expect("Failed to read file");
    let mut lexer = parsing::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap_or_else(|e| panic!("{}", e));
    let mut parser = parsing::parser::Parser::new(tokens, &source);
    let mut ast = parser.parse().unwrap_or_else(|e| panic!("{}", e));
    let mut checker = semantic_checker::SemanticChecker::new(&mut ast, &source);
    checker.check().unwrap_or_else(|e| panic!("{}", e));
    let ir_module = ir::lower::IrModuleBuilder::new(&ast).lower();
    println!("{:#?}", ir_module);
}
