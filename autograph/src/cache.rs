use notify;
use notify::Watcher;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Weak};
use std::time::Duration;

#[derive(Debug)]
pub struct CacheCell<T: ?Sized> {
    id: String,
    inner: T,
}

impl<T> CacheCell<T> {
    pub fn new(id: String, inner: T) -> CacheCell<T> {
        CacheCell { id, inner }
    }
}

pub struct Cache {
    cached_objects: RefCell<HashMap<String, Box<CacheCell<Any>>>>,
    fs_watcher: RefCell<notify::RecommendedWatcher>,
    fs_events: Receiver<notify::DebouncedEvent>,
}

impl ::std::fmt::Debug for Cache {
    fn fmt(&self, _f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        Ok(())
    }
}

pub enum ReloadReason {
    Initial,
    FileCreated,
    FileModified,
    FileRemoved,
}

impl Cache {
    pub fn new() -> Cache {
        // setup notification channel
        let (tx, rx) = channel();
        let watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();

        Cache {
            cached_objects: RefCell::new(HashMap::new()),
            fs_events: rx,
            fs_watcher: RefCell::new(watcher),
        }
    }

    pub fn process_filesystem_events(&self) {
        // go through all filesystem events and see if one concerns an object in the cache
        for ev in self.fs_events.try_iter() {
            debug!("watch event: {:?}", ev);
        }
    }

    /// replaces existing elements (does not invalidate previous versions,
    /// as they are allocated with Arc)
    /// returns a copy of the element
    pub fn add<T>(&self, path: String, obj: T) -> T
    where
        T: Any + Clone,
    {
        let mut hash = self.cached_objects.borrow_mut();
        let newobj = Box::new(CacheCell::new(path.to_owned(), obj.clone()));
        hash.insert(path, newobj);
        obj
    }

    pub fn add_and_watch<T, F>(&self, path: String, f: F) -> Option<T>
    where
        T: Any + Clone,
        F: Fn(&str, ReloadReason) -> Option<T>,
    {
        let result = f(&path, ReloadReason::Initial).map(|val| self.add(path.clone(), val));
        // setup watch
        self.fs_watcher
            .borrow_mut()
            .watch(&path, notify::RecursiveMode::NonRecursive);
        result
    }

    pub fn get_or<T, F>(&self, path: &str, f: F) -> Option<T>
    where
        T: Any + Clone,
        F: FnOnce() -> T,
    {
        let mut hash = self.cached_objects.borrow_mut();
        // if the hashmap doesn't have an entry, call f(), box the returned value, add it to the hash,
        // downcast it to the concrete type and return it
        let obj = hash
            .entry(path.to_owned())
            .or_insert_with(|| Box::new(CacheCell::new(path.to_owned(), f().clone())));

        obj.inner.downcast_ref::<T>().map(|v| v.clone())
    }

    pub fn get<T>(&self, path: &str) -> Option<T>
    where
        T: Any + Clone,
    {
        // noice
        self.cached_objects
            .borrow()
            .get(path)
            .and_then(|obj| obj.inner.downcast_ref::<T>().map(|v| v.clone()))
    }
}
