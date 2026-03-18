use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub status: TaskStatus,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plan {
    pub goal: String,
    pub tasks: Vec<Task>,
    pub current_task_index: Option<usize>,
}

impl Plan {
    #[allow(dead_code)]
    pub fn new(goal: &str) -> Self {
        Self {
            goal: goal.to_string(),
            tasks: Vec::new(),
            current_task_index: None,
        }
    }

    // pub fn next_task(&mut self) -> Option<&mut Task> {
    //     if let Some(idx) = self.current_task_index {
    //         if idx < self.tasks.len() - 1 {
    //             self.current_task_index = Some(idx + 1);
    //             return Some(&mut self.tasks[idx + 1]);
    //         }
    //         return None;
    //     } else {
    //         // Start from first task
    //         if !self.tasks.is_empty() {
    //             self.current_task_index = Some(0);
    //             return Some(&mut self.tasks[0]);
    //         }
    //         return None;
    //     }
    // }
}
