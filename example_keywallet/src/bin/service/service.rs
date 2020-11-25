use example_keywallet::LockState;
use example_keywallet::LookupAttribute;
use example_keywallet::Secret;

pub struct Item {
    id: String,
    attrs: Vec<LookupAttribute>,
    secret: Secret,
    lock_state: LockState,

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

pub enum UnlockError {
    NotFound,
}

impl SecretService {
    fn next_id(&mut self) -> String {
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
    pub fn unlock_collection(&mut self, id: &str) -> Result<(), UnlockError> {
        let coll = self.collections.iter_mut().find(|s| s.id.eq(id));
        if let Some(mut coll) = coll {
            coll.lock_state = LockState::Unlocked;
            Ok(())
        } else {
            Err(UnlockError::NotFound)
        }
    }
    pub fn unlock_item(&mut self, id: &str) -> Result<(), UnlockError> {
        let item = self
            .collections
            .iter_mut()
            .find_map(|s| s.items.iter_mut().find(|i| i.id.eq(id)));
        if let Some(mut item) = item {
            item.lock_state = LockState::Unlocked;
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
    pub fn get_secret(&self, id: &str) -> Result<Secret, GetSecretError> {
        let item = self
            .collections
            .iter()
            .find_map(|s| s.items.iter().find(|i| i.id.eq(id)));
        if let Some(item) = item {
            Ok(item.secret.clone())
        } else {
            Err(GetSecretError::NotFound)
        }
    }
    pub fn set_secret(&mut self, id: &str, secret: Secret) -> Result<(), SetSecretError> {
        let item = self
            .collections
            .iter_mut()
            .find_map(|s| s.items.iter_mut().find(|i| i.id.eq(id)));
        if let Some(item) = item {
            item.secret = secret;
            Ok(())
        } else {
            Err(SetSecretError::NotFound)
        }
    }
    pub fn get_secrets(&self, ids: &[&str]) -> Result<Vec<(String, Secret)>, GetSecretError> {
        let items: Result<Vec<_>, _> = ids
            .iter()
            .map(|id| self.get_secret(id).map(|secret| (id.to_string(), secret)))
            .collect();
        let items = items?;
        Ok(items)
    }
}

impl Collection {
    pub fn create_item(
        &mut self,
        secret: &[u8],
        attrs: &[LookupAttribute],
    ) -> Result<String, CreateItemError> {
        Ok("ABCD".into())
    }
    pub fn delete_item(&mut self, id: &str) -> Result<(), DeleteItemError> {
        Ok(())
    }
    pub fn search_items(&self, attrs: &[LookupAttribute]) -> Vec<&Item> {
        vec![]
    }
}
