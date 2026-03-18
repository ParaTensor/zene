use xtrace_client::{Client, XtraceLayer};
use tracing::error;

pub fn init_xtrace(endpoint: &str, token: &str) -> Option<XtraceLayer> {
    match Client::new(endpoint, token) {
        Ok(client) => Some(XtraceLayer::new(client)),
        Err(e) => {
            error!("Xtrace: Failed to initialize client: {}", e);
            None
        }
    }
}
