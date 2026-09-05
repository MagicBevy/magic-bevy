use std::sync::Mutex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompilerLockMode {
    Free,       
    EditorLock,
}

pub struct EngineCompilerState {
    pub lock_mode: Mutex<CompilerLockMode>,
    pub request_reload: Mutex<bool>,
}

impl EngineCompilerState {
    pub fn global() -> &'static Self {
        static STATE: OnceLock<EngineCompilerState> = OnceLock::new();
        STATE.get_or_init(|| EngineCompilerState {
            lock_mode: Mutex::new(CompilerLockMode::Free),
            request_reload: Mutex::new(false),
        })
    }

    pub fn lock_watcher(&self) {
        if let Ok(mut mode) = self.lock_mode.lock() {
            *mode = CompilerLockMode::EditorLock;
        }
    }

    pub fn unlock_watcher(&self) {
        if let Ok(mut mode) = self.lock_mode.lock() {
            *mode = CompilerLockMode::Free;
        }
    }

    pub fn trigger_recompile(&self) {
        if let Ok(mut req) = self.request_reload.lock() {
            *req = true;
        }
    }
}