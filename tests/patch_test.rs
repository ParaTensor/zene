use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use zene::engine::context::ContextEngine;
use zene::engine::tools::ToolManager;

fn setup_test_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn test_patch_exact_match() -> Result<()> {
    let path = Path::new("target/test_exact.txt");
    let content = "line1\nline2\nline3\nline4";
    setup_test_file(path, content)?;

    let original = "line2\nline3";
    let new = "line2_modified\nline3_modified";

    let context_engine = Arc::new(Mutex::new(ContextEngine::new(false)?));
    let tm = ToolManager::new(None, context_engine);
    tm.apply_patch(path.to_str().unwrap(), original, new, None)?;

    let result = fs::read_to_string(path)?;
    assert_eq!(result, "line1\nline2_modified\nline3_modified\nline4");
    Ok(())
}

#[test]
fn test_patch_fuzzy_whitespace() -> Result<()> {
    let path = Path::new("target/test_fuzzy.txt");
    // Original file has 4 spaces indentation
    let content = "function test() {\n    let a = 1;\n    let b = 2;\n    return a + b;\n}";
    setup_test_file(path, content)?;

    // Patch provided by Agent has 2 spaces indentation (mismatch)
    let original = "  let a = 1;\n  let b = 2;";
    let new = "    let a = 10;\n    let b = 20;";

    let context_engine = Arc::new(Mutex::new(ContextEngine::new(false)?));
    let tm = ToolManager::new(None, context_engine);
    tm.apply_patch(path.to_str().unwrap(), original, new, None)?;

    let result = fs::read_to_string(path)?;
    // The indentation of the new block should be preserved as provided in `new`
    let expected = "function test() {\n    let a = 10;\n    let b = 20;\n    return a + b;\n}";
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_patch_start_line_hint() -> Result<()> {
    let path = Path::new("target/test_start_line.txt");
    // File with duplicate content
    let content = "block1\nstart\nend\n\nblock2\nstart\nend";
    setup_test_file(path, content)?;

    // We want to replace the second "start\nend" block
    let original = "start\nend";
    let new = "start_2\nend_2";
    
    // Hint: it's around line 6
    let context_engine = Arc::new(Mutex::new(ContextEngine::new(false)?));
    let tm = ToolManager::new(None, context_engine);
    tm.apply_patch(path.to_str().unwrap(), original, new, Some(6))?;

    let result = fs::read_to_string(path)?;
    assert_eq!(result, "block1\nstart\nend\n\nblock2\nstart_2\nend_2");
    Ok(())
}

#[test]
fn test_patch_failure_not_found() {
    let path = Path::new("target/test_fail.txt");
    let content = "line1\nline2";
    setup_test_file(path, content).unwrap();

    let original = "line3"; // Does not exist
    let new = "line3_new";

    let context_engine = Arc::new(Mutex::new(ContextEngine::new(false).unwrap()));
    let tm = ToolManager::new(None, context_engine);
    let result = tm.apply_patch(path.to_str().unwrap(), original, new, None);
    assert!(result.is_err());
}
