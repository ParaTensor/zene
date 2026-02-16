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

    pub fn next_task(&mut self) -> Option<&mut Task> {
        // Find the first Pending task
        for (i, task) in self.tasks.iter_mut().enumerate() {
            if matches!(task.status, TaskStatus::Pending) {
                self.current_task_index = Some(i);
                task.status = TaskStatus::InProgress;
                return Some(task);
            }
        }
        None
    }
}
