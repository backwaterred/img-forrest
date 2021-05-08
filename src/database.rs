use std::cmp::Eq;
use std::hash::Hash;
use std::fmt::Display;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::fs::{ self, File, };
use std::collections::{ HashMap, HashSet, };

use bincode;
#[allow(unused_imports)]
use serde::{ Deserialize, Serialize };
use serde::de::DeserializeOwned;

/// This is the primary API for the module.
pub trait Table<K,V>
{
    fn set(&mut self, k: K, v: V) -> Option<V>;
    fn get(&mut self, k: &K) -> Option<&V>;
    fn contains_key(&self, k: &K) -> bool;
    fn remove(&mut self, k: &K) -> Option<V>;
}

/// A Hash(map)-Backed-Table with no persistant storage
pub struct MemCache<K,V>(HashMap<K,V>);

impl<K,V> MemCache<K,V>
    where K: Eq + Hash
{
    pub fn new() -> Self
    {
        Self(HashMap::<K,V>::new())
    }
}

impl<K,V> Table<K,V> for MemCache<K,V>
    where K: Eq + Hash
{
    fn set(&mut self, k: K, v: V) -> Option<V>
    {
        self.0.insert(k,v)
    }

    fn get(&mut self, k: &K) -> Option<&V>
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
    where K: Eq + Hash + Display,
          V: DeserializeOwned + Serialize
{
    base_path: PathBuf,
    disk_update_required: HashSet<K>,
    cache: HashMap<K,V>,
}

impl<K,V> DiskCache<K,V>
    where K: Eq + Hash + Display,
          V: DeserializeOwned + Serialize
{
    pub fn new(base_path: PathBuf) -> Self
    {
        DiskCache
        {
            base_path,
            disk_update_required: HashSet::<K>::new(),
            cache: HashMap::<K,V>::new(),
        }
    }

    pub fn persist(&self) -> Result<(), Box<dyn Error>>
    {
        for k in self.disk_update_required.iter()
        {
            if let Some(v) = self.cache.get(k)
            {
                // Update record
                let fpath = self.make_path(k);
                let mut f = File::create(fpath)?;

                let fdata = bincode::serialize(v)?;
                f.write(&fdata)?;
            }
            else if self.is_on_disk(k)
            {
                // Remove record
                fs::remove_file(self.make_path(k))?;
            }
            else
            {
                // A double remove perhaps?
            }
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

    fn get_from_disk(&self, k: &K) -> Result<Box<V>, Box<dyn Error>>
    {
        let path = self.make_path(k);

        let file = File::open(path)?;
        let data: V = bincode::deserialize_from(file)?;

        Ok(Box::new(data))
    }
}

impl<K,V> Table<K,V> for DiskCache<K,V>
    where K: Clone + Display + Eq + Hash,
          V: DeserializeOwned + Serialize
{
    fn set(&mut self, k: K, v: V) -> Option<V>
    {
        self.disk_update_required.insert(k.clone());
        self.cache.insert(k,v)
    }

    fn get(&mut self, k: &K) -> Option<&V>
    {
        if self.cache.contains_key(k)
        {
            self.cache.get(k)
        }
        else if let Ok(boxed_v) = self.get_from_disk(k)
        {
            self.cache.insert((*k).clone(), *boxed_v);
            self.cache.get(k)
        }
        else
        {
            None
        }
    }

    fn contains_key(&self, k: &K) -> bool
    {
        self.cache.contains_key(k) ||
        self.is_on_disk(k)
    }

    fn remove(&mut self, k: &K) -> Option<V>
    {
        self.disk_update_required.insert((*k).clone());
        self.cache.remove(k)
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[derive(Clone, Hash, Eq, PartialEq)]
    struct TestKey { k: String }

    impl Display for TestKey
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "{}", self.k)
        }
    }

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct TestVal { v: String }

    const TEST_STR: &str = "~~~~😸 + 🦀 = 🎇~~~~";

   // #[test]
   // fn create_test_records()
   // {
   //     // For creating any needed test resources uncomment this fn
   //     let mut base_path = std::env::current_dir().unwrap();
   //     base_path.push("database-test-db");

   //     let mut dc = DiskCache::<TestKey, TestVal>::new(base_path);
   //     for n in 0..8
   //     {
   //         let key = TestKey { k: String::from(format!("multiple-record-{}", n)) };
   //         let val = TestVal { v: String::from(TEST_STR)};
   //         dc.set(key, val);
   //     }

   //     dc.persist().unwrap();

   //     panic!()
   // }
   //

    fn scrub_a_dub(record: &PathBuf)
    {
        // Clean up created resources if present
        match fs::remove_file(record)
        {
            Ok(_) => {},
            Err(_) => {},
        };

       assert!(!record.exists());
    }

    #[test]
    fn add_contains_rm_dc_sanity()
    {
        let foo = String::from("this-record-not-persisted");
        let bar = String::from("Hello World!");
        let mut dc = DiskCache::new(PathBuf::new());

        assert!(!dc.contains_key(&foo));
        assert_eq!(dc.set(foo.clone(), bar.clone()), None);
        assert!(dc.contains_key(&foo));
        assert_eq!(dc.remove(&foo), Some(bar));
        assert!(!dc.contains_key(&foo));
    }

    #[test]
    fn dc_contains_key_finds_records_on_disk()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("database-test-db");
        let dc = DiskCache::<TestKey, TestVal>::new(base_path);

        let key = TestKey { k: String::from("this-record-exists") };

        assert!(!dc.cache.contains_key(&key));
        assert!(dc.contains_key(&key));
    }

    #[test]
    fn dc_adds_extant_record()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("database-test-db");
        let mut dc = DiskCache::<TestKey, TestVal>::new(base_path);

        let key = TestKey { k: String::from("this-record-exists") };

        assert!(!dc.cache.contains_key(&key));
        assert_eq!(
            &TestVal { v: String::from(TEST_STR) },
            dc.get(&key).unwrap()
        );

        assert_eq!(1, dc.cache.len());
        assert!(dc.cache.contains_key(&key));
    }

    #[test]
    fn dc_adds_multiple_extant_record()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("database-test-db");
        let mut dc = DiskCache::<TestKey, TestVal>::new(base_path);

        let mut keys = Vec::with_capacity(8);
        for n in 0..8
        {
            let key = TestKey { k: String::from(format!("multiple-record-{}", n)) };
            keys.push(key);
        }

        for key in &keys
        {
            dc.get(key);
        }

        assert_eq!(8, dc.cache.len());

        for key in &keys
        {
            assert!(dc.contains_key(&key));
            assert_eq!(
                &TestVal { v: String::from(TEST_STR) },
                dc.get(&key).unwrap()
            );
        }
    }

    #[test]
    fn dc_doesnt_add_non_extant_record()
    {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("database-test-db");
        let key = TestKey { k: String::from("this-record-does-not-exist") };
        let mut dc = DiskCache::<TestKey, TestVal>::new(base_path);
        dc.get(&key);

        assert!(!dc.contains_key(&key));
    }

    #[test]
    fn dc_persists_new_records_to_disk()
    {
        let key = String::from("a-new-record");
        let val = String::from("bar");
        let mut test_db = std::env::current_dir().unwrap();
        test_db.push("database-test-db");

        let mut record = test_db.clone();
        let mut dc = DiskCache::new(test_db);
        record.push(&key);

        // Records should not be present before starting
        // or else this test is meaningless
        scrub_a_dub(&record);

        dc.set(key.clone(), val.clone());
        dc.persist().unwrap();

        // Records should be present after persist operation
        assert!(record.exists());

        // Clean Up
        scrub_a_dub(&record);
    }

    #[test]
    fn dc_persist_removes_from_disk()
    {
        let key = String::from("a-record-on-disk");
        let val = String::from("bar");
        let mut test_db = std::env::current_dir().unwrap();
        test_db.push("database-test-db");

        let mut record = test_db.clone();
        let mut dc = DiskCache::new(test_db);
        record.push(&key);

        // Set initial value on disk
        scrub_a_dub(&record);
        dc.set(key.clone(), val.clone());
        dc.persist().unwrap();

        dc.cache.clear();

        // Record should not be present on disk
        // after remove & persist
        dc.remove(&key);
        dc.persist().unwrap();
        assert!(!record.exists());

        // Clean Up
        scrub_a_dub(&record);
    }

    #[test]
    fn dc_persist_updates_existing_record()
    {
        let key = String::from("a-record-to-update");
        let val0 = String::from("bar");
        let val1 = String::from("baz");
        let mut test_db = std::env::current_dir().unwrap();
        test_db.push("database-test-db");

        let mut record = test_db.clone();
        let mut dc = DiskCache::new(test_db);
        record.push(&key);

        // Records should not be present before starting
        scrub_a_dub(&record);

        // Set initial value on disk
        dc.set(key.clone(), val0.clone());
        dc.persist().unwrap();
        assert!(record.exists());

        dc.cache.clear();

        // Set updated value on disk
        dc.remove(&key);
        dc.set(key.clone(), val1.clone());
        dc.persist().unwrap();

        dc.cache.clear();

        assert!(record.exists());
        assert_eq!(
            &val1,
            dc.get(&key).unwrap()
        );

        // Clean Up
        scrub_a_dub(&record);
    }
}
