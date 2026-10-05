pub mod profile;
pub mod systems;

pub use systems::{EditorSystem, SystemManager};

use magic_core::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn register_all_packages(registry: &mut EditorRegistry) {
    magic_pkg_editor::register_all_packages(registry);
}
