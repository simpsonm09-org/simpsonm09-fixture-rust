//! The store port the service depends on and the in-memory adapter.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::item::domain::Item;

/// The three items seeded on startup so the API has something to return.
pub const DEFAULT_ITEMS: [(&str, &str); 3] = [
    ("Widget", "A small widget"),
    ("Gadget", "A handy gadget"),
    ("Gizmo", "A clever gizmo"),
];

/// The store port. It speaks domain types, so the service never sees a store
/// record and the adapter can be swapped without touching business logic.
pub trait ItemStore: Send + Sync {
    fn find_all(&self) -> Vec<Item>;
    fn find_by_id(&self, id: i64) -> Option<Item>;
    fn save(&self, item: Item) -> Item;
    /// Replaces the item with `id` under one lock. Returns `None` when no item
    /// has that id, so a concurrent delete cannot race the write.
    fn update(&self, id: i64, name: String, description: Option<String>) -> Option<Item>;
    /// Removes the item with `id` under one lock, returning whether one existed.
    fn delete(&self, id: i64) -> bool;
}

/// In-memory store adapter. It converges to the seeded state on restart, so a
/// restart discards every item the caller created and restores the seed.
pub struct InMemoryItemStore {
    inner: Mutex<Inner>,
}

struct Inner {
    items: BTreeMap<i64, Item>,
    next_id: i64,
}

impl InMemoryItemStore {
    /// Builds a store holding the three dev seeds.
    pub fn new() -> Self {
        let store = Self {
            inner: Mutex::new(Inner {
                items: BTreeMap::new(),
                next_id: 1,
            }),
        };
        store.seed_defaults();
        store
    }

    /// Replaces every record with the dev seeds.
    pub fn seed_defaults(&self) {
        self.seed(
            DEFAULT_ITEMS
                .iter()
                .map(|(name, description)| ((*name).to_string(), Some((*description).to_string()))),
        );
    }

    /// Replaces every record with the given seeds.
    pub fn seed<I>(&self, seeds: I)
    where
        I: IntoIterator<Item = (String, Option<String>)>,
    {
        let mut inner = self.lock();
        inner.items.clear();
        inner.next_id = 1;
        for (name, description) in seeds {
            let id = inner.next_id;
            inner.next_id += 1;
            inner.items.insert(
                id,
                Item {
                    id: Some(id),
                    name,
                    description,
                },
            );
        }
    }

    /// Empties the store. Used by tests.
    pub fn clear(&self) {
        let mut inner = self.lock();
        inner.items.clear();
        inner.next_id = 1;
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("item store lock poisoned")
    }
}

impl Default for InMemoryItemStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemStore for InMemoryItemStore {
    fn find_all(&self) -> Vec<Item> {
        self.lock().items.values().cloned().collect()
    }

    fn find_by_id(&self, id: i64) -> Option<Item> {
        self.lock().items.get(&id).cloned()
    }

    fn save(&self, item: Item) -> Item {
        let mut inner = self.lock();
        let id = match item.id {
            Some(id) => id,
            None => {
                let id = inner.next_id;
                inner.next_id += 1;
                id
            }
        };
        let stored = Item {
            id: Some(id),
            name: item.name,
            description: item.description,
        };
        inner.items.insert(id, stored.clone());
        stored
    }

    fn update(&self, id: i64, name: String, description: Option<String>) -> Option<Item> {
        let mut inner = self.lock();
        let entry = inner.items.get_mut(&id)?;
        entry.name = name;
        entry.description = description;
        Some(entry.clone())
    }

    fn delete(&self, id: i64) -> bool {
        self.lock().items.remove(&id).is_some()
    }
}
