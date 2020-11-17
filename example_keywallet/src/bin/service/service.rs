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

pub struct Session {
    id: String,
}

#[derive(Default)]
pub struct SecretService {
    collections: Vec<Collection>,
    sessions: Vec<Session>,
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
pub enum OpenSessionError {}
#[derive(Debug)]
pub enum CloseSessionError {}

#[derive(Debug)]
pub enum GetSecretError {
    Locked,
}
#[derive(Debug)]
pub enum SetSecretError {
    Locked,
}

pub enum UnlockError {}

impl SecretService {
    pub fn create_collection(&mut self, label: &str) -> Result<String, CreateCollectionError> {
        Ok("ABCD".into())
    }
    pub fn delete_collection(&mut self, id: &str) -> Result<(), DeleteCollectionError> {
        Ok(())
    }
    pub fn unlock_collection(&mut self, id: &str) -> Result<(), UnlockError> {
        Ok(())
    }
    pub fn unlock_item(&mut self, id: &str) -> Result<(), UnlockError> {
        Ok(())
    }
    pub fn open_session(&mut self, alg: &str) -> Result<String, OpenSessionError> {
        Ok("ABCD".into())
    }
    pub fn close_session(&mut self, id: &str) -> Result<String, CloseSessionError> {
        Ok("ABCD".into())
    }
    pub fn get_secret(&self, id: &str) -> Result<Secret, GetSecretError> {
        Ok(Secret {
            value: vec![],
            params: vec![],
            content_type: "txt/plain".into(),
        })
    }
    pub fn set_secret(&mut self, id: &str, secret: &[u8]) -> Result<(), GetSecretError> {
        Ok(())
    }
    pub fn get_secrets(&self, ids: &[&str]) -> Result<Vec<(String, Secret)>, SetSecretError> {
        Ok(vec![])
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
