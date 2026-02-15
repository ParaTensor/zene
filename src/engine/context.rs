use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct ContextEngine {
    #[allow(dead_code)]
    parser: Parser,
}

impl ContextEngine {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        // Default to Rust for now, in reality we'd swap languages dynamically
        parser.set_language(tree_sitter_rust::language())?;
        Ok(Self { parser })
    }

    /// L1: Scan project structure
    pub fn scan_project(&self, root: &Path) -> Vec<String> {
        let mut files = Vec::new();
        for entry in WalkBuilder::new(root).build().flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                files.push(entry.path().display().to_string());
            }
        }
        files
    }

    /// L2: Analyze file for definitions (Structs, Functions)
    #[allow(dead_code)]
    pub fn analyze_definitions(&mut self, source_code: &str) -> Result<Vec<String>> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or(anyhow::anyhow!("Failed to parse"))?;

        // Simple query to find function definitions
        let query_str = "(function_item name: (identifier) @function)";
        let query = Query::new(tree_sitter_rust::language(), query_str)?;
        let mut cursor = QueryCursor::new();

        let mut definitions = Vec::new();
        for m in cursor.matches(&query, tree.root_node(), source_code.as_bytes()) {
            for capture in m.captures {
                let range = capture.node.byte_range();
                let name = &source_code[range];
                definitions.push(name.to_string());
            }
        }

        Ok(definitions)
    }
}
