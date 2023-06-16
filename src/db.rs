use std::collections::HashMap;

use bytes::Bytes;

use crate::utils::now;

pub struct Db {
    values: HashMap<String, Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    data: Bytes,
    ttl: Option<u64>,
    inserted_at: u128,
}

impl Db {
    pub fn new() -> Db {
        Db {
            values: HashMap::new(),
        }
    }

    /// Returns the previous entry if it's to be overwritten
    pub fn set(&mut self, key: String, data: Bytes, ttl: Option<u64>) -> Option<Bytes> {
        let previous_entry = if let Some(entry) = self.values.get(&key) {
            Some(entry.data.clone())
        } else {
            None
        };
        let entry = Entry {
            data,
            inserted_at: now(),
            ttl,
        };
        self.values.insert(key, entry);

        previous_entry
    }

    pub fn get(&mut self, key: &str) -> Option<Bytes> {
        if let Some(entry) = self.values.get(key) {
            if let Some(ttl) = entry.ttl {
                if now() > entry.inserted_at + (ttl as u128) {
                    self.values.remove(key);
                    return None;
                }
            }
            self.values.get(key).map(|entry| entry.data.clone())
        } else {
            None
        }
    }
}
