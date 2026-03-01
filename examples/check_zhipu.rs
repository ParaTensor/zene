use llm_providers;

fn main() {
    let providers = llm_providers::get_providers_data();
    
    for name in ["deepseek", "zhipu", "minimax"] {
        if let Some(p) = providers.get(name) {
            println!("\nProvider: {}", name);
            println!("  Endpoints keys: {:?}", p.endpoints.keys().collect::<Vec<_>>());
            for (k, e) in p.endpoints.entries() {
                println!("    Region: {}, URL: {}", k, e.base_url);
            }
        } else {
            println!("\nProvider {} not found", name);
        }
    }
}
