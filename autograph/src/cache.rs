use std::any::{Any};
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::{Arc, Weak};
use std::fs::File;
use notify;
use notify::Watcher;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

#[derive(Debug)]
pub struct CacheCell<T: ?Sized> {
    cache: Weak<Cache>,
    id: String,
    inner: T,
}

impl<T> CacheCell<T> {
    pub fn new(cache: Weak<Cache>, id: String, inner: T) -> CacheCell<T> {
        CacheCell { cache, id, inner }
    }
}


pub struct Cache {
    cached_objects: RefCell<HashMap<String, Box<CacheCell<Any>>>>,
    fs_watcher: RefCell<notify::RecommendedWatcher>,
    fs_events: Receiver<notify::DebouncedEvent>,
}

impl ::std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        Ok(())
    }
}

impl Cache {
    pub fn new() -> Arc<Cache> {
        // setup notification channel
        let (tx, rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();

        Arc::new(Cache {
            cached_objects: RefCell::new(HashMap::new()),
            fs_events: rx,
            fs_watcher: RefCell::new(watcher),
        })
    }

    pub fn process_filesystem_events(&self) {
        // go through all filesystem events and see if one concerns an object in the cache
        for ev in self.fs_events.try_iter() {
            debug!("watch event: {:?}", ev);
        }
    }
}

pub enum ReloadReason {
    Initial,
    FileCreated,
    FileModified,
    FileRemoved,
}

pub trait CacheTrait
{
    fn add<T>(&self, path: String, obj: T) -> T
    where
        T: Any + Clone;

    fn get_or<T, F>(&self, path: &str, f: F) -> Option<T>
    where
        T: Any + Clone,
        F: FnOnce() -> T;

    fn get<T>(&self, path: &str) -> Option<T>
    where
        T: Any + Clone;

    fn add_and_watch<T, F>(&self, path: String, f: F) -> Option<T>
    where
        T: Any + Clone,
        F: Fn(&str, ReloadReason) -> Option<T>;
}


/// Proposition: the cache handles the file watching
/// add_and_watch(url, Fn(url, change) -> T) -> Cached<T>
///
/// Proposition: how about returning a value instead of an Arc?
/// Sometimes it's more convenient
/// The user can still wrap it into an Arc
/// Hot-reload: just query the cache again for an updated value
impl CacheTrait for Arc<Cache> {
    /// replaces existing elements (does not invalidate previous versions,
    /// as they are allocated with Arc)
    /// returns a copy of the element
    fn add<T>(&self, path: String, obj: T) -> T
    where
        T: Any + Clone,
    {
        let mut hash = self.cached_objects.borrow_mut();
        let newobj = Box::new(CacheCell::new(
            Arc::downgrade(self),
            path.to_owned(),
            obj.clone(),
        ));
        hash.insert(path, newobj);
        obj
    }

    fn add_and_watch<T, F>(&self, path: String, f: F) -> Option<T>
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

    fn get_or<T, F>(&self, path: &str, f: F) -> Option<T>
    where
        T: Any + Clone,
        F: FnOnce() -> T,
    {
        let mut hash = self.cached_objects.borrow_mut();
        // if the hashmap doesn't have an entry, call f(), box the returned value, add it to the hash,
        // downcast it to the concrete type and return it
        let obj = hash.entry(path.to_owned()).or_insert_with(|| {
            Box::new(CacheCell::new(
                Arc::downgrade(self),
                path.to_owned(),
                f().clone(),
            ))
        });

        obj.inner.downcast_ref::<T>().map(|v| v.clone())
    }

    fn get<T>(&self, path: &str) -> Option<T>
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
