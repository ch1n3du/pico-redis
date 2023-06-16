use std::collections::{btree_map::Values, HashMap};

use bytes::Bytes;

pub struct Db {
    values: HashMap<String, Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    data: Bytes,
}

impl Db {
    pub fn new() -> Db {
        Db {
            values: HashMap::new(),
        }
    }

    /// Returns the previous entry if it's to be overwritten
    pub fn set(&mut self, key: String, data: Bytes) -> Option<Bytes> {
        let previous_entry = if let Some(entry) = self.values.get(&key) {
            Some(entry.data.clone())
        } else {
            None
        };
        let entry = Entry { data };
        self.values.insert(key, entry);

        previous_entry
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        self.values.get(key).map(|entry| entry.data.clone())
    }
}
