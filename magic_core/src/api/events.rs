pub type EventCallback = Box<dyn Fn() + Send + Sync + 'static>;

#[derive(Default)]
pub struct EditorEventRegistry {
    pub on_compile_start: Vec<EventCallback>,
    pub on_compile_success: Vec<EventCallback>,
}

impl EditorEventRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.on_compile_start.clear();
        self.on_compile_success.clear();
    }

    pub fn listen_compile_start<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.on_compile_start.push(Box::new(callback));
    }

    pub fn listen_compile_success<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.on_compile_success.push(Box::new(callback));
    }
}