use example_keywallet::LockState;
use example_keywallet::LookupAttribute;
use example_keywallet::Secret;

#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub lock_state: LockState,
    attrs: Vec<LookupAttribute>,
    secret: Secret,

    // properties from API
    label: String,
    created: u64,
    modified: u64,
}

pub struct Collection {
    id: String,
    lock_state: LockState,
    items: Vec<Item>,

    // properties from API
    label: String,
    created: u64,
    modified: u64,
}

pub enum SessionAlg {
    Plain,
}

pub struct Session {
    id: String,
    alg: SessionAlg,
}

#[derive(Default)]
pub struct SecretService {
    collections: Vec<Collection>,
    sessions: Vec<Session>,
    id_gen: u64,
}

#[derive(Debug)]
pub enum CreateItemError {
    Locked,
}
#[derive(Debug)]
pub enum DeleteItemError {
    Locked,
    NotFound,
}
#[derive(Debug)]
pub enum CreateCollectionError {
    Locked,
}
#[derive(Debug)]
pub enum DeleteCollectionError {
    Locked,
    NotFound,
}
#[derive(Debug)]
pub enum OpenSessionError {
    UnsupportedAlg(String),
}
#[derive(Debug)]
pub enum CloseSessionError {
    NotFound,
}

#[derive(Debug)]
pub enum GetSecretError {
    Locked,
    NotFound,
}
#[derive(Debug)]
pub enum SetSecretError {
    Locked,
    NotFound,
}

#[derive(Debug)]
pub enum UnlockError {
    NotFound,
}

impl SecretService {
    pub fn next_id(&mut self) -> String {
        let id = self.id_gen.to_string();
        self.id_gen += 1;
        id
    }

    pub fn create_collection(&mut self, label: &str) -> Result<String, CreateCollectionError> {
        let coll = Collection {
            id: self.next_id(),
            lock_state: LockState::Locked,
            items: vec![],
            label: label.into(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        };

        let path = format!("/org/freedesktop/secrets/collection/{}", coll.id);

        Ok(path)
    }
    pub fn delete_collection(&mut self, id: &str) -> Result<(), DeleteCollectionError> {
        let idx = self
            .collections
            .iter()
            .enumerate()
            .find(|(_idx, s)| s.id.eq(id))
            .map(|(idx, _)| idx);
        if let Some(idx) = idx {
            self.collections.remove(idx);
            Ok(())
        } else {
            Err(DeleteCollectionError::NotFound)
        }
    }
    pub fn delete_item(&mut self, col_id: &str, item_id: &str) -> Result<(), DeleteItemError> {
        let col = self.collections.iter_mut().find(|s| s.id.eq(col_id));
        if let Some(col) = col {
            col.delete_item(item_id)
        } else {
            Err(DeleteItemError::NotFound)
        }
    }
    pub fn unlock_collection(&mut self, id: &str) -> Result<(), UnlockError> {
        let coll = self.collections.iter_mut().find(|s| s.id.eq(id));
        if let Some(mut coll) = coll {
            coll.lock_state = LockState::Unlocked;
            Ok(())
        } else {
            Err(UnlockError::NotFound)
        }
    }
    pub fn unlock_item(&mut self, col_id: &str, item_id: &str) -> Result<(), UnlockError> {
        let item = self
            .collections
            .iter_mut()
            .find(|col| col.id.eq(col_id))
            .map(|col| col.items.iter_mut().find(|i| i.id.eq(item_id)));
        if let Some(Some(mut item)) = item {
            item.lock_state = LockState::Unlocked;
            Ok(())
        } else {
            Err(UnlockError::NotFound)
        }
    }
    pub fn lock_collection(&mut self, id: &str) -> Result<(), UnlockError> {
        let coll = self.collections.iter_mut().find(|s| s.id.eq(id));
        if let Some(mut coll) = coll {
            coll.lock_state = LockState::Locked;
            Ok(())
        } else {
            Err(UnlockError::NotFound)
        }
    }
    pub fn lock_item(&mut self, col_id: &str, item_id: &str) -> Result<(), UnlockError> {
        let item = self
            .collections
            .iter_mut()
            .find(|col| col.id.eq(col_id))
            .map(|col| col.items.iter_mut().find(|i| i.id.eq(item_id)));
        if let Some(Some(mut item)) = item {
            item.lock_state = LockState::Locked;
            Ok(())
        } else {
            Err(UnlockError::NotFound)
        }
    }
    pub fn open_session(&mut self, alg: &str) -> Result<String, OpenSessionError> {
        if alg != "plain" {
            Err(OpenSessionError::UnsupportedAlg(alg.into()))
        } else {
            let session = Session {
                alg: SessionAlg::Plain,
                id: self.next_id(),
            };
            let path = format!("/org/freedesktop/secrets/session/{}", session.id);
            self.sessions.push(session);
            Ok(path)
        }
    }
    pub fn close_session(&mut self, id: &str) -> Result<(), CloseSessionError> {
        let idx = self
            .sessions
            .iter()
            .enumerate()
            .find(|(_idx, s)| s.id.eq(id))
            .map(|(idx, _)| idx);
        if let Some(idx) = idx {
            self.sessions.remove(idx);
            Ok(())
        } else {
            Err(CloseSessionError::NotFound)
        }
    }
    pub fn get_secret(&self, col_id: &str, item_id: &str) -> Result<Secret, GetSecretError> {
        let col = self
            .collections
            .iter()
            .find(|col| col.id.eq(col_id))
            .unwrap();
        let item = col.items.iter().find(|i| i.id.eq(item_id));
        if let Some(item) = item {
            Ok(item.secret.clone())
        } else {
            Err(GetSecretError::NotFound)
        }
    }
    pub fn set_secret(
        &mut self,
        col_id: &str,
        item_id: &str,
        secret: Secret,
    ) -> Result<(), SetSecretError> {
        let col = self
            .collections
            .iter_mut()
            .find(|col| col.id.eq(col_id))
            .unwrap();
        let item = col.items.iter_mut().find(|i| i.id.eq(item_id));
        if let Some(item) = item {
            item.secret = secret;
            Ok(())
        } else {
            Err(SetSecretError::NotFound)
        }
    }

    pub fn search_items<'a>(&'a self, attrs: &'a [LookupAttribute]) -> Vec<(&'a str, &'a Item)> {
        self.collections
            .iter()
            .map(|coll| {
                coll.search_items(attrs)
                    .into_iter()
                    .map(|item| (coll.id.as_str(), item))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
    pub fn get_collection(&self, id: &str) -> Option<&Collection> {
        self.collections.iter().find(|coll| coll.id.eq(id))
    }
    pub fn get_collection_mut(&mut self, id: &str) -> Option<&mut Collection> {
        self.collections.iter_mut().find(|coll| coll.id.eq(id))
    }
}

impl Collection {
    pub fn create_item(
        &mut self,
        id: String,
        secret: &example_keywallet::messages::Secret,
        attrs: &[LookupAttribute],
        _replace: bool,
    ) -> Result<String, CreateItemError> {
        let item = Item {
            id: id.clone(),
            lock_state: LockState::Unlocked,
            attrs: attrs.to_vec(),
            secret: Secret {
                params: secret.params.clone(),
                value: secret.params.clone(),
                content_type: secret.content_type.clone(),
            },
            label: "Label".to_owned(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        };
        self.items.push(item);
        Ok(id)
    }
    pub fn delete_item(&mut self, id: &str) -> Result<(), DeleteItemError> {
        let idx = self
            .items
            .iter()
            .enumerate()
            .find(|(_idx, s)| s.id.eq(id))
            .map(|(idx, _)| idx);
        if let Some(idx) = idx {
            self.items.remove(idx);
            Ok(())
        } else {
            Err(DeleteItemError::NotFound)
        }
    }

    pub fn search_items<'a>(&'a self, attrs: &'a [LookupAttribute]) -> Vec<&'a Item> {
        self.items
            .iter()
            .filter(|item| attrs.iter().any(|attr| item.attrs.contains(attr)))
            .collect()
    }
}
