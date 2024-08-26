use crate::utils::truncated_str;

pub struct StatusTask {
    task_type: TaskType,
    what: String,
}

#[derive(Clone, Copy)]
enum TaskType {
    Add,
    Delete,
    Move,
    Open,
    ChangeCategory,
}

impl StatusTask {
    pub fn new_add(what: impl Into<String>) -> Self {
        StatusTask {
            task_type: TaskType::Add,
            what: what.into(),
        }
    }

    pub fn new_del(what: impl Into<String>) -> Self {
        StatusTask {
            task_type: TaskType::Delete,
            what: what.into(),
        }
    }

    pub fn new_move(what: impl Into<String>) -> Self {
        StatusTask {
            task_type: TaskType::Move,
            what: what.into(),
        }
    }

    pub fn new_category(what: impl Into<String>) -> Self {
        StatusTask {
            task_type: TaskType::ChangeCategory,
            what: what.into(),
        }
    }

    pub fn new_open(what: impl Into<String>) -> Self {
        StatusTask {
            task_type: TaskType::Open,
            what: what.into(),
        }
    }

    pub fn success_str(&self) -> String {
        let truncated = truncated_str(&self.what, 60);

        match self.task_type {
            TaskType::Add => format!("Added {truncated}"),
            TaskType::Delete => format!("Deleted {truncated}"),
            TaskType::Move => format!("Moved {truncated}"),
            TaskType::Open => format!("Opened {truncated}"),
            TaskType::ChangeCategory => {
                if truncated.is_empty() {
                    "Categories cleared!".to_string()
                } else {
                    format!("Category set to {truncated}!")
                }
            }
        }
    }

    pub fn failure_str(&self) -> String {
        let truncated = truncated_str(&self.what, 60);

        match self.task_type {
            TaskType::Add => format!("Error adding {truncated}"),
            TaskType::Delete => format!("Error deleting {truncated}"),
            TaskType::Move => format!("Error moving to {truncated}"),
            TaskType::Open => format!("Error opening {truncated}"),
            TaskType::ChangeCategory => format!("Error changing category to {truncated}"),
        }
    }

    pub fn loading_str(&self) -> String {
        let truncated = truncated_str(&self.what, 60);

        match self.task_type {
            TaskType::Add => format!("Adding {truncated}"),
            TaskType::Delete => format!("Deleting {truncated}"),
            TaskType::Move => format!("Moving {truncated}"),
            TaskType::Open => format!("Opening {truncated}"),
            TaskType::ChangeCategory => format!("Changing category to {truncated}"),
        }
    }
}
