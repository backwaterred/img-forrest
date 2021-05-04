use std::path::PathBuf;
use std::fs::{
    self,
    DirBuilder, DirEntry,
};
use std::collections::HashMap;

use serde::Serialize;

/// This is the primary API for the module.
pub trait Table<K,V>
{
    fn set(&mut self, k: K, v: V) -> Option<V>;
    fn get(&self, k: &K) -> Option<&V>;
    fn contains_key(&self, k: &K) -> bool;
    fn remove(&mut self, k: &K) -> Option<V>;
}

/// A Hash(map)-Backed-Table with no persistant storage
pub struct HBT<K,V>(HashMap<K,V>);

impl<K,V> HBT<K,V>
    where K: std::cmp::Eq + std::hash::Hash
{
    pub fn new() -> Self
    {
        Self(HashMap::<K,V>::new())
    }
}

impl<K,V> Table<K,V> for HBT<K,V>
    where K: std::cmp::Eq + std::hash::Hash
{
    fn set(&mut self, k: K, v: V) -> Option<V>
    {
        self.0.insert(k,v)
    }

    fn get(&self, k: &K) -> Option<&V>
    {
        self.0.get(k)
    }

    fn contains_key(&self, k: &K) -> bool
    {
        self.0.contains_key(k)
    }

    fn remove(&mut self, k: &K) -> Option<V>
    {
        self.0.remove(k)
    }
}

/// A Lazy-Populated Cache of items persisted by the system disk
pub struct DiskCache<K,V>
    where K: std::cmp::Eq + std::hash::Hash,
          V: Serialize
{
    base_path: PathBuf,
    cache: HashMap<K,V>,
}

impl<K,V> DiskCache<K,V>
    where K: std::cmp::Eq + std::hash::Hash,
          V: Serialize
{
    pub fn new(base_path: PathBuf) -> Self
    {
        DiskCache
        {
            base_path,
            cache: HashMap::<K,V>::new(),
        }
    }

    pub fn persist(&self)
    {
        unimplemented!()
    }
}

impl<K,V> Table<K,V> for DiskCache<K,V>
    where K: std::cmp::Eq + std::hash::Hash,
          V: Serialize
{
    fn set(&mut self, k: K, v: V) -> Option<V>
    {
        self.cache.insert(k,v)
    }

    fn get(&self, k: &K) -> Option<&V>
    {
        self.cache.get(k)
    }

    fn contains_key(&self, k: &K) -> bool
    {
        self.cache.contains_key(k)
    }

    fn remove(&mut self, k: &K) -> Option<V>
    {
        self.cache.remove(k)
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[derive(Hash, Eq, PartialEq)]
    struct TestKey { k: String }

    #[derive(Serialize)]
    struct TestVal { v: String }

    #[test]
    fn add_contains_rm_dc_sanity()
    {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let mut dc = DiskCache::new(PathBuf::new());

        assert!(!dc.contains_key(&foo));
        assert_eq!(dc.set(foo.clone(), bar.clone()), None);
        assert!(dc.contains_key(&foo));
        assert_eq!(dc.remove(&foo), Some(bar));
        assert!(!dc.contains_key(&foo));
    }

    #[test]
    fn dc_finds_extant_record()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("test-db");
        let key = TestKey { k: String::from("this-record-exists") };
        let dc = DiskCache::<TestKey, TestVal>::new(base_path);

        assert!(dc.contains_key(&key));
    }

    #[test]
    fn dc_doesnt_find_non_extant_record()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("test-db");
        let key = TestKey { k: String::from("this-record-does-not-exist") };
        let dc = DiskCache::<TestKey, TestVal>::new(base_path);

        assert!(!dc.contains_key(&key));
    }

    #[test]
    fn dc_doesnt_add_extant_record_with_garbage_data()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("test-db");
        let key = TestKey { k: String::from("this-record-contains-garbage-data") };
        let dc = DiskCache::<TestKey, TestVal>::new(base_path);

        assert!(!dc.contains_key(&key));
    }

    #[test]
    fn persist_creates_records_on_disk()
    {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let mut test_db = std::env::current_dir().unwrap();
        test_db.push("test-db/");
        assert!(test_db.is_dir());

        let mut dc = DiskCache::new(test_db.clone());

        // Records should not be present before starting
        test_db.push("foo");
        assert!(!test_db.exists());

        dc.set(foo.clone(), bar.clone());
        dc.persist();

        // Records should be present after pesist operation
        assert!(test_db.exists());
    }

}
