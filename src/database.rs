use std::error::Error;
use std::io::{ Read, Write };
use std::path::PathBuf;
use std::fs::{
    DirBuilder, DirEntry,
    File,
};
use std::collections::HashMap;

use bincode;
use serde::{ Deserialize, Serialize };

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
    where K: std::cmp::Eq + std::hash::Hash + std::fmt::Display,
          V: Deserialize + Serialize
{
    base_path: PathBuf,
    cache: HashMap<K,V>,
}

impl<K,V> DiskCache<K,V>
    where K: std::cmp::Eq + std::hash::Hash + std::fmt::Display,
          V: Deserialize + Serialize
{
    pub fn new(base_path: PathBuf) -> Self
    {
        DiskCache
        {
            base_path,
            cache: HashMap::<K,V>::new(),
        }
    }

    pub fn persist(&self) -> Result<(), Box<dyn Error>>
    {
        for (k,v) in self.cache.iter()
        {
            let fpath = self.make_path(k);
            let mut f = File::create(fpath)?;

            let fdata = bincode::serialize(&v)?;
            f.write(&fdata)?;
        }

        Ok(())
    }

    fn make_path(&self, k: &K) -> PathBuf
    {
        let mut path = self.base_path.clone();
        path.push(k.to_string());

        path
    }

    fn is_on_disk(&self, k: &K) -> bool
    {
        self.make_path(k).exists()
    }

    fn get_from_disk(&mut self, k: &K) -> Result<V, Box<dyn Error>>
    {
        let path = self.make_path(k);
        let mut buf = Vec::new();

        let file = File::open(path)?;
        let len = file.read(&mut buf)?;
        let data: V = bincode::deserialize(&buf[..len])?;

        Ok(data)
    }
}

impl<K,V> Table<K,V> for DiskCache<K,V>
    where K: std::cmp::Eq + std::hash::Hash + std::fmt::Display,
          V: Deserialize + Serialize
{
    fn set(&mut self, k: K, v: V) -> Option<V>
    {
        self.cache.insert(k,v)
    }

    fn get(&self, k: &K) -> Option<&V>
    {
        if self.contains_key(k)
        {
            self.cache.get(k)
        }
        else if self.is_on_disk(k)
        {
            if let Ok(v) = self.get_from_disk(k)
            {
                self.set(k.clone(), v);
                self.cache.get(k)
            }
            else
            {
                None
            }
        }
        else
        {
            None
        }
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

    use std::fs;

    #[derive(Hash, Eq, PartialEq)]
    struct TestKey { k: String }

    impl std::fmt::Display for TestKey
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "{}", self.k)
        }
    }

    #[derive(Serialize)]
    struct TestVal { v: String }

//    #[test]
//    fn create_test_records()
//    {
//        let mut base_path = std::env::current_dir().unwrap();
//        base_path.push("test-db");
//        let key = TestKey { k: String::from("this-record-exists") };
//        let val = TestVal { v: String::from("~~~~😸 + 🦀 = 🎇~~~~")};
//        let mut dc = DiskCache::<TestKey, TestVal>::new(base_path);
//        dc.set(key, val);
//
//        dc.persist().unwrap();
//
//        panic!()
//    }

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
    fn dc_adds_extant_record()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("test-db");
        let key = TestKey { k: String::from("this-record-exists") };
        let dc = DiskCache::<TestKey, TestVal>::new(base_path);

        assert!(dc.contains_key(&key));
    }

    #[test]
    fn dc_doesnt_add_non_extant_record()
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
    fn dc_persists_records_to_disk()
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
        dc.persist().unwrap();

        // Records should be present after pesist operation
        assert!(test_db.exists());

        // Clean up created resources
        fs::remove_file(test_db).unwrap();
    }

}
