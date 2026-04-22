use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::parsing::{
    ast::{Stmt, StmtNode},
    lexer::Lexer,
    parser::Parser,
};

pub fn expand_includes(
    stmts: Vec<StmtNode>,
    current_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Vec<StmtNode> {
    let mut result = Vec::new();

    for stmt in stmts {
        match stmt.stmt {
            Stmt::Include(path) => {
                let full_path = current_path.parent().unwrap().join(&path);

                if visited.contains(&full_path) {
                    continue; // or error if you prefer
                }
                visited.insert(full_path.clone());

                let source = std::fs::read_to_string(&full_path).expect("failed to read include");

                // re-run pipeline (lexer + parser ONLY)
                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize().unwrap();
                let mut parser = Parser::new(tokens, &source);
                let ast = parser.parse().unwrap();

                let expanded = expand_includes(ast, &full_path, visited);

                result.extend(expanded);
            }
            Stmt::Block(inner) => {
                let new_block = expand_includes(inner, current_path, visited);
                result.push(StmtNode::new(Stmt::Block(new_block), stmt.span));
            }
            _ => result.push(stmt),
        }
    }

    result
}
