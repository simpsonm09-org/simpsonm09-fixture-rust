//! Item business logic. It works in the domain `Item` and depends on the
//! `ItemStore` port, so it never sees a transport DTO.

use std::sync::Arc;

use crate::item::domain::{Item, ItemError};
use crate::item::store::ItemStore;

/// The service the API layer drives.
pub struct ItemService {
    store: Arc<dyn ItemStore>,
}

impl ItemService {
    pub fn new(store: Arc<dyn ItemStore>) -> Self {
        Self { store }
    }

    pub fn list_items(&self) -> Vec<Item> {
        self.store.find_all()
    }

    pub fn get_item(&self, id: i64) -> Result<Item, ItemError> {
        self.store.find_by_id(id).ok_or(ItemError::NotFound(id))
    }

    pub fn create_item(&self, name: String, description: Option<String>) -> Item {
        self.store.save(Item {
            id: None,
            name,
            description,
        })
    }

    pub fn update_item(
        &self,
        id: i64,
        name: String,
        description: Option<String>,
    ) -> Result<Item, ItemError> {
        if !self.store.exists_by_id(id) {
            return Err(ItemError::NotFound(id));
        }
        Ok(self.store.save(Item {
            id: Some(id),
            name,
            description,
        }))
    }

    pub fn delete_item(&self, id: i64) -> Result<(), ItemError> {
        if !self.store.exists_by_id(id) {
            return Err(ItemError::NotFound(id));
        }
        self.store.delete_by_id(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// An in-memory fake so the service tests never touch the real adapter.
    #[derive(Default)]
    struct FakeStore {
        items: Mutex<Vec<Item>>,
        next_id: Mutex<i64>,
    }

    impl FakeStore {
        fn with(items: Vec<Item>) -> Self {
            Self {
                items: Mutex::new(items),
                next_id: Mutex::new(1),
            }
        }
    }

    impl ItemStore for FakeStore {
        fn find_all(&self) -> Vec<Item> {
            self.items.lock().unwrap().clone()
        }

        fn find_by_id(&self, id: i64) -> Option<Item> {
            self.items
                .lock()
                .unwrap()
                .iter()
                .find(|item| item.id == Some(id))
                .cloned()
        }

        fn save(&self, item: Item) -> Item {
            let mut items = self.items.lock().unwrap();
            let id = match item.id {
                Some(id) => id,
                None => {
                    let mut next = self.next_id.lock().unwrap();
                    let id = *next;
                    *next += 1;
                    id
                }
            };
            let stored = Item {
                id: Some(id),
                ..item
            };
            items.retain(|item| item.id != Some(id));
            items.push(stored.clone());
            stored
        }

        fn delete_by_id(&self, id: i64) {
            self.items.lock().unwrap().retain(|item| item.id != Some(id));
        }

        fn exists_by_id(&self, id: i64) -> bool {
            self.items.lock().unwrap().iter().any(|item| item.id == Some(id))
        }
    }

    fn service(items: Vec<Item>) -> ItemService {
        ItemService::new(Arc::new(FakeStore::with(items)))
    }

    #[test]
    fn lists_every_item() {
        let service = service(vec![Item {
            id: Some(1),
            name: "Widget".into(),
            description: None,
        }]);
        assert_eq!(service.list_items().len(), 1);
    }

    #[test]
    fn returns_an_item_by_id() {
        let service = service(vec![Item {
            id: Some(1),
            name: "Widget".into(),
            description: None,
        }]);
        assert_eq!(service.get_item(1).unwrap().name, "Widget");
    }

    #[test]
    fn raises_when_an_item_is_missing() {
        let service = service(vec![]);
        assert_eq!(service.get_item(9), Err(ItemError::NotFound(9)));
    }

    #[test]
    fn creates_an_item_through_the_store() {
        let service = service(vec![]);
        let created = service.create_item("Gadget".into(), Some("A handy gadget".into()));
        assert_eq!(created.id, Some(1));
        assert_eq!(created.name, "Gadget");
    }

    #[test]
    fn replaces_an_existing_item() {
        let service = service(vec![Item {
            id: Some(7),
            name: "Old".into(),
            description: None,
        }]);
        let updated = service.update_item(7, "Renamed".into(), None).unwrap();
        assert_eq!(updated.id, Some(7));
        assert_eq!(updated.name, "Renamed");
    }

    #[test]
    fn refuses_to_update_a_missing_item() {
        let service = service(vec![]);
        assert_eq!(
            service.update_item(404, "Nope".into(), None),
            Err(ItemError::NotFound(404))
        );
    }

    #[test]
    fn deletes_an_existing_item() {
        let service = service(vec![Item {
            id: Some(1),
            name: "Widget".into(),
            description: None,
        }]);
        service.delete_item(1).unwrap();
        assert!(service.list_items().is_empty());
    }

    #[test]
    fn refuses_to_delete_a_missing_item() {
        let service = service(vec![]);
        assert_eq!(service.delete_item(404), Err(ItemError::NotFound(404)));
    }
}
