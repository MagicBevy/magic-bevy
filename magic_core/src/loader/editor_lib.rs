use crate::{EditorSystem, SystemManager};
use std::path::Path;

pub struct PackageManager {
    active_systems: SystemManager,
    current_lib: Option<libloading::Library>,
}

impl PackageManager {
    pub fn new() -> Self {
        Self {
            active_systems: SystemManager::new(),
            current_lib: None,
        }
    }

    pub fn reload_editor(&mut self) -> Result<(), String> {
        self.active_systems.reload_systems();
        self.current_lib = None;

        #[cfg(target_os = "linux")]
        let lib_path = "./target/debug/libmagic_editor.so";
        #[cfg(target_os = "windows")]
        let lib_path = "./target/debug/magic_editor.dll";
        #[cfg(target_os = "macos")]
        let lib_path = "./target/debug/libmagic_editor.dylib";

        if !Path::new(lib_path).exists() {
            return Err(format!("Library file not found at: {}", lib_path));
        }

        unsafe {
            match libloading::Library::new(lib_path) {
                Ok(lib) => {
                    type CreateFn = fn() -> Box<dyn EditorSystem>;
                    
                    if let Ok(constructor) = lib.get::<CreateFn>(b"create_system\0") {
                        let editor_workspace = constructor();
                        self.active_systems.add_system(editor_workspace);
                        
                        self.current_lib = Some(lib);
                        Ok(())
                    } else {
                        Err("Could not find 'create_system' symbol in editor library.".to_string())
                    }
                }
                Err(e) => Err(format!("Failed to load library: {:?}", e)),
            }
        }
    }

    pub fn systems_mut(&mut self) -> &mut SystemManager {
        &mut self.active_systems
    }
}