// Hot reload state preservation
// 
// This module allows preserving state across hot reloads.
// Useful for keeping database connections, caches, etc. alive.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    static ref STATE_STORE: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

/// Store state that persists across hot reloads
pub fn preserve_state<T: Any + Send + Sync + 'static>(key: impl Into<String>, value: T) {
    let mut store = STATE_STORE.write().unwrap();
    store.insert(key.into(), Box::new(value));
}

/// Retrieve preserved state from previous reload
pub fn restore_state<T: Any + Send + Sync + Clone + 'static>(key: &str) -> Option<T> {
    let store = STATE_STORE.read().unwrap();
    store.get(key).and_then(|v| v.downcast_ref::<T>()).cloned()
}

/// Check if state exists
pub fn has_state(key: &str) -> bool {
    let store = STATE_STORE.read().unwrap();
    store.contains_key(key)
}

/// Clear all preserved state
pub fn clear_state() {
    let mut store = STATE_STORE.write().unwrap();
    store.clear();
}

/// Macro for easy state preservation
#[macro_export]
macro_rules! preserve {
    ($key:expr, $value:expr) => {
        $crate::hot_reload_state::preserve_state($key, $value)
    };
}

/// Macro for easy state restoration with default
#[macro_export]
macro_rules! restore {
    ($key:expr, $default:expr) => {
        $crate::hot_reload_state::restore_state($key).unwrap_or($default)
    };
    ($key:expr) => {
        $crate::hot_reload_state::restore_state($key)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_preservation() {
        preserve_state("test_key", 42);
        assert_eq!(restore_state::<i32>("test_key"), Some(42));
    }

    #[test]
    fn test_state_missing() {
        assert_eq!(restore_state::<String>("missing"), None);
    }
}
