use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor};
use crate::engine::memory::MemoryManager;

pub struct ContextEngine {
    #[allow(dead_code)]
    parser: Parser,
    pub memory: Option<MemoryManager>,
}

impl ContextEngine {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        // Default to Rust for now, in reality we'd swap languages dynamically
        parser.set_language(tree_sitter_rust::language())?;
        
        let root = std::env::current_dir()?;
        let memory = match MemoryManager::new(&root) {
            Ok(m) => Some(m),
            Err(e) => {
                tracing::warn!("Failed to initialize MemoryManager: {}", e);
                None
            }
        };

        Ok(Self { parser, memory })
    }

    /// L1: Scan project structure (limited depth)
    /// Only returns directories and files at the top level or within a limited depth
    pub fn list_files(&self, root: &Path, depth: Option<usize>) -> Vec<String> {
        let mut files = Vec::new();
        let mut builder = WalkBuilder::new(root);
        if let Some(d) = depth {
            builder.max_depth(Some(d));
        }
        
        for entry in builder.build().flatten() {
            if entry.path() == root {
                continue;
            }
            // Strip prefix for cleaner output
            let path_str = if let Ok(stripped) = entry.path().strip_prefix(root) {
                stripped.display().to_string()
            } else {
                entry.path().display().to_string()
            };
            
            // Append "/" to directories for clarity
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                files.push(format!("{}/", path_str));
            } else {
                files.push(path_str);
            }
        }
        files
    }

    /// Search for a pattern in the project (ripgrep-like)
    pub fn search_code(&self, root: &Path, pattern: &str) -> Result<Vec<String>> {
        use grep::regex::RegexMatcher;
        use grep::searcher::Searcher;
        use grep::searcher::sinks::UTF8;

        let matcher = RegexMatcher::new(pattern)?;
        let mut matches = Vec::new();

        for result in WalkBuilder::new(root).build() {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().map_or(true, |ft| !ft.is_file()) {
                continue;
            }

            let path = entry.path();
            let relative_path = path.strip_prefix(root).unwrap_or(path).display().to_string();

            let _ = Searcher::new().search_path(
                &matcher,
                path,
                UTF8(|lnum, line| {
                    let match_str = format!("{}:{}: {}", relative_path, lnum, line.trim());
                    matches.push(match_str);
                    Ok(true)
                }),
            );

            // Limit results to prevent context overflow
            if matches.len() > 100 {
                matches.push("... (too many matches, truncated)".to_string());
                break;
            }
        }

        if matches.is_empty() {
            Ok(vec!["No matches found.".to_string()])
        } else {
            Ok(matches)
        }
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

    pub fn index_project(&mut self, root: &Path) -> Result<String> {
        if let Some(memory) = &mut self.memory {
            memory.index_project(root)
        } else {
             Err(anyhow::anyhow!("Memory engine not initialized"))
        }
    }
}
