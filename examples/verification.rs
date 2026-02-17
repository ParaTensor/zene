use zene::engine::tools::ToolManager;
use std::collections::HashMap;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Python Execution Verification...");

    // 1. Setup Session Envs
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "CorrectValue".to_string());
    println!("✅ Session Environment Variables prepared.");

    // 2. Create a test python script
    let script_path = "verify_test.py";
    let script_content = r#"
import os
import sys

print(f"Python Executable: {sys.executable}")
print(f"TEST_VAR: {os.environ.get('TEST_VAR')}")
"#;
    fs::write(script_path, script_content)?;
    println!("✅ Created test python script: {}", script_path);

    // 3. Run Python using ToolManager
    println!("🚀 Running python script via ToolManager::run_python...");
    let args = vec![];
    let tm = ToolManager::new(None);
    let output = tm.run_python(script_path, &args, &env_vars).await?;

    println!("--- Script Output ---");
    println!("{}", output);
    println!("---------------------");

    // 4. Verification Logic
    if output.contains("TEST_VAR: CorrectValue") {
        println!("✅ SUCCESS: Environment variable injected correctly.");
    } else {
        println!("❌ FAILURE: Environment variable NOT found in output.");
    }

    if output.contains(".venv") {
         println!("✅ SUCCESS: Script ran inside .venv (path contained '.venv').");
    } else {
         println!("⚠️ WARNING: Script might not be running in .venv (check output path).");
    }

    // Clean up
    let _ = fs::remove_file(script_path);
    
    // 5. Test User Confirmation (Mocking UI not easy here, skipping)
    
    Ok(())
}
