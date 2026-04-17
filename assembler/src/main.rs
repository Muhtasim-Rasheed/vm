use std::io::Write;

mod lower;
mod parser;

fn main() {
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
    let out = parser::parse_instructions(&source).expect("Failed to parse instructions");
    if !out.0.is_empty() {
        panic!("Unparsed input remaining: {}", out.0);
    }
    let ast = out.1;
    let symbols = lower::build_symbol_table(&ast);
    let mut symbols_vec: Vec<_> = symbols.iter().collect();
    symbols_vec.sort_by_key(|(_, addr)| *addr);
    for (name, addr) in &symbols_vec {
        println!("{:#010x}: {}", addr, name);
    }
    let lowered = lower::lower_instructions(&ast, &symbols).expect("Failed to lower instructions");
    let mut file = std::fs::File::create(out_path).expect("Failed to create output file");
    let mut writer = std::io::BufWriter::new(&mut file);
    let mut pc = 0u32;

    for instr in lowered {
        match instr {
            lower::LoweredInstruction::Instruction(instr) => {
                println!("{:#010x}: {}", pc, instr);
                pc += 8;

                let bytes = u64::from(instr).to_le_bytes();
                writer
                    .write_all(&bytes)
                    .expect("Failed to write instruction");
            }
            lower::LoweredInstruction::Directive(directive) => match directive {
                parser::Directive::Data(items) => {
                    for item in items {
                        match item {
                            parser::DataItem::Byte(byte) => {
                                writer.write_all(&[byte]).expect("Failed to write byte");
                            }
                            parser::DataItem::Bytes(bytes) => {
                                writer.write_all(&bytes).expect("Failed to write bytes");
                            }
                        }
                    }
                }
                parser::Directive::Word(exprs) => {
                    for expr in exprs {
                        let value = lower::resolve_expr(expr, &symbols)
                            .expect("Failed to evaluate expression");
                        writer
                            .write_all(&value.to_le_bytes())
                            .expect("Failed to write word");
                    }
                }
                _ => unreachable!(),
            },
        }
    }
}
