use magic_core::mgprint;

use super::system::EditorSystem;

pub struct SystemManager {
    active_systems: Vec<Box<dyn EditorSystem>>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            active_systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, mut system: Box<dyn EditorSystem>) {
        system.on_enable();
        self.active_systems.push(system);
    }

    pub fn systems_mut(&mut self) -> &mut Vec<Box<dyn EditorSystem>> {
        &mut self.active_systems
    }

    pub fn reload_systems(&mut self) {
        mgprint!("SystemManager", "Preparing for hot-reload...");
        
        for sys in &mut self.active_systems {
            sys.on_disable();
        }
        
        self.active_systems.clear();
        mgprint!("SystemManager", "Old systems successfully cleared and memory freed.");
    }
}

impl Drop for SystemManager {
    fn drop(&mut self) {
        for sys in &mut self.active_systems {
            sys.on_disable();
        }
        mgprint!("SystemManager", "All editor systems have been successfully disabled.");
    }
}