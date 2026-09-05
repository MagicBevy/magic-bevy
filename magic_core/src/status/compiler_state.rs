#[derive(Debug, Clone, PartialEq)]
pub enum CompileStatus {
    Idle,
    Compiling { current_file: String, progress: usize },
    Success,
    Error(String),
}

pub struct CompilerStateManager {
    pub status: CompileStatus,
}

impl CompilerStateManager {
    pub fn new() -> Self {
        Self {
            status: CompileStatus::Idle,
        }
    }

    pub fn set_compiling(&mut self, file: &str, progress: usize) {
        self.status = CompileStatus::Compiling {
            current_file: file.to_string(),
            progress,
        };
    }

    pub fn set_success(&mut self) {
        self.status = CompileStatus::Success;
    }

    pub fn set_error(&mut self, err: &str) {
        self.status = CompileStatus::Error(err.to_string());
    }
}