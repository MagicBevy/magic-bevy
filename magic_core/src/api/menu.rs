use std::collections::BTreeMap;

use crate::*;

pub type CommandCallback = Box<dyn Fn() + Send + Sync + 'static>;

pub struct MenuCommand {
    pub label: String,
    pub callback: CommandCallback,
}

pub struct MenuRegistry {
    pub menus: BTreeMap<String, Vec<MenuCommand>>,
}

impl MenuRegistry {
    pub fn new() -> Self {
        Self {
            menus: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, menu_name: &str, label: &str, callback: CommandCallback) {
        let command = MenuCommand {
            label: label.to_string(),
            callback,
        };
        self.menus
            .entry(menu_name.to_string())
            .or_insert_with(Vec::new)
            .push(command);
    }

    pub fn clear(&mut self) {
        self.menus.clear();
        mgprint!("MenuRegistry", "All top editor menus have been successfully reset and cleared.");
    }
}
