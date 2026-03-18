use crate::engine::ui::UserInterface;
use std::sync::{Arc, Mutex};

pub struct MockUserInterface {
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl MockUserInterface {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }

    pub fn show_message(&self, message: &str) {
        self.messages.lock().unwrap().push(format!("MSG: {}", message));
    }

    pub fn show_error(&self, error: &str) {
        self.messages.lock().unwrap().push(format!("ERR: {}", error));
    }
}

impl UserInterface for MockUserInterface {
    fn confirm_execution(&self, _tool_name: &str, _args: &str) -> bool {
        true
    }
}
