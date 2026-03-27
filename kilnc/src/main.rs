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
    let tokens = lexer.tokenize().expect("Failed to tokenize source");
    let mut parser = parsing::parser::Parser::new(tokens, &source);
    let ast = parser.parse().expect("Failed to parse source");
    println!("{:#?}", ast);
}
