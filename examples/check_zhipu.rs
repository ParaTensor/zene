use llm_providers;

fn main() {
    let providers = llm_providers::get_providers_data();
    if let Some(p) = providers.get("minimax") {
        println!("Minimax Endpoints: {:?}", p.endpoints.keys().collect::<Vec<_>>());
        for (k, e) in p.endpoints.entries() {
            println!("  Region: {}, URL: {}", k, e.base_url);
        }
    } else {
        println!("Minimax not found");
    }
}
