use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::Cache;
use ramd_db::storage::Storage;

pub struct InMemoryCache<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    cache: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
    tombstone: RwLock<HashSet<Vec<u8>>>,
    storage: Arc<S>,
}

impl<S> InMemoryCache<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            cache: RwLock::new(BTreeMap::new()),
            tombstone: RwLock::new(HashSet::new()),
            storage,
        }
    }

    fn read_cache(&self) -> eyre::Result<RwLockReadGuard<BTreeMap<Vec<u8>, Vec<u8>>>> {
        let cache = self
            .cache
            .read()
            .map_err(|err| eyre::eyre!(err.to_string()))?;

        Ok(cache)
    }

    fn write_cache(&self) -> eyre::Result<RwLockWriteGuard<BTreeMap<Vec<u8>, Vec<u8>>>> {
        let cache = self
            .cache
            .write()
            .map_err(|err| eyre::eyre!(err.to_string()))?;

        Ok(cache)
    }

    fn read_tombstone(&self) -> eyre::Result<RwLockReadGuard<HashSet<Vec<u8>>>> {
        let tombstone = self
            .tombstone
            .read()
            .map_err(|err| eyre::eyre!(err.to_string()))?;

        Ok(tombstone)
    }

    fn write_tombstone(&self) -> eyre::Result<RwLockWriteGuard<HashSet<Vec<u8>>>> {
        let tombstone = self
            .tombstone
            .write()
            .map_err(|err| eyre::eyre!(err.to_string()))?;

        Ok(tombstone)
    }
}

impl<S> Storage<Vec<u8>, Vec<u8>> for InMemoryCache<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    fn has(&self, key: Vec<u8>) -> eyre::Result<bool> {
        if self.read_cache()?.contains_key(&key) {
            return Ok(true);
        }

        match self.storage.get_opt(key.clone())? {
            Some(value) => {
                self.set(key, value.clone())?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn get(&self, key: Vec<u8>) -> eyre::Result<Vec<u8>> {
        if let Some(value) = self.read_cache()?.get(&key) {
            return Ok(value.clone());
        }

        let value = self.storage.get(key.clone())?;
        self.set(key, value.clone())?;
        Ok(value)
    }

    fn get_opt(&self, key: Vec<u8>) -> eyre::Result<Option<Vec<u8>>> {
        if let Some(value) = self.read_cache()?.get(&key) {
            return Ok(Some(value.clone()));
        }

        match self.storage.get_opt(key.clone())? {
            Some(value) => {
                self.set(key, value.clone())?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> eyre::Result<()> {
        self.write_tombstone()?.remove(&key);
        self.write_cache()?.insert(key, value);
        Ok(())
    }

    fn delete(&self, key: Vec<u8>) -> eyre::Result<()> {
        self.write_cache()?.remove(&key);
        self.write_tombstone()?.insert(key);
        Ok(())
    }
}

impl<S> Cache for InMemoryCache<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    fn commit(&self) -> eyre::Result<()> {
        for (key, value) in self.read_cache()?.iter() {
            self.storage.set(key.clone(), value.clone())?;
        }

        for key in self.read_tombstone()?.iter() {
            self.storage.delete(key.clone())?;
        }

        Ok(())
    }
}
