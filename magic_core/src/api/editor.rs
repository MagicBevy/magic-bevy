use crate::api::events::EditorEventRegistry;
use crate::api::menu::MenuRegistry;

pub struct EditorRegistry<'a> {
    pub menu: &'a mut MenuRegistry,
    pub events: &'a mut EditorEventRegistry,
}