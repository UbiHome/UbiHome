use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::internal::sensors::UbiComponent;

#[derive(Clone, Debug, PartialEq)]
pub enum EntityState {
    Switch(bool),
    BinarySensor(bool),
    Sensor(f32),
    Number(f32),
    TextSensor(String),
    Light {
        state: bool,
        brightness: Option<f32>,
        red: Option<f32>,
        green: Option<f32>,
        blue: Option<f32>,
    },
}

/// Read-only handle to the global entity/state cache. Every platform module
/// receives one of these via [`crate::Module::run`]. There is no way to
/// mutate the underlying data through this type - only [`StateStoreWriter`],
/// which is never handed to modules, can do that.
#[derive(Clone)]
pub struct StateStore {
    components: Arc<Vec<UbiComponent>>,
    entity_states: Arc<RwLock<HashMap<String, EntityState>>>,
}

impl StateStore {
    pub fn components(&self) -> &[UbiComponent] {
        &self.components
    }

    pub fn get(&self, key: &str) -> Option<EntityState> {
        self.entity_states
            .read()
            .expect("state store lock poisoned")
            .get(key)
            .cloned()
    }

    pub fn get_all(&self) -> HashMap<String, EntityState> {
        self.entity_states
            .read()
            .expect("state store lock poisoned")
            .clone()
    }
}

/// Owns write access to the global entity/state cache. Only the main
/// application (`src/commands/run.rs`) should construct this - platform
/// modules only ever see the read-only [`StateStore`] handed to them.
/// `Clone` is for the main application's own internal tasks (which each
/// write a different subset of entities) to share write access - it does
/// not leak write access to platform modules, which never receive this type.
#[derive(Clone)]
pub struct StateStoreWriter {
    entity_states: Arc<RwLock<HashMap<String, EntityState>>>,
}

impl StateStoreWriter {
    /// The component list is topology: known before any module runs and
    /// fixed for the lifetime of the process, so it is set once here rather
    /// than mutated later.
    pub fn new(components: Vec<UbiComponent>) -> (StateStoreWriter, StateStore) {
        let entity_states = Arc::new(RwLock::new(HashMap::new()));
        let store = StateStore {
            components: Arc::new(components),
            entity_states: entity_states.clone(),
        };
        (StateStoreWriter { entity_states }, store)
    }

    pub fn set(&self, key: String, state: EntityState) {
        self.entity_states
            .write()
            .expect("state store lock poisoned")
            .insert(key, state);
    }
}
