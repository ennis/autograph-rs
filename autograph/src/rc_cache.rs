use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use std::cell::RefCell;
use std::mem;
use std::rc::{Rc,Weak};
use std::marker::PhantomData;
use std::ops::Deref;
use std::fs::File;
use notify;
use notify::Watcher;
use std::sync::mpsc::{Receiver,channel};
use std::time::Duration;

#[derive(Debug)]
pub struct CacheCell<T: ?Sized>
{
    cache: Weak<Cache>,
    id: String,
    inner: T
}

impl<T> CacheCell<T>
{
    pub fn new(cache: Weak<Cache>, id: String, inner: T) -> CacheCell<T>
    {
        CacheCell {
            cache, id, inner
        }
    }
}

///
/// Wrapper around a cached object of type T
/*#[derive(Clone,Debug)]
pub struct Cached<T: 'static>
{
    ptr: Rc<CacheBox<Any>>,
    _phantom: PhantomData<T>
}


impl<T> Cached<T>
{
    fn new(cache: Weak<Cache>, id: String, value: T) -> Cached<T> {
        Self::from_any(Rc::new( CacheBox::new( cache,id, value))).unwrap()
    }

    fn from_any(ptr: Rc<CacheBox<Any>>) -> Option<Cached<T>> {
        match ptr.inner.downcast_ref::<T>() {
            Some(_) => Some(Cached { ptr, _phantom: PhantomData }),
            None => None
        }
    }

    /// Check if there is an updated version of the cached
    /// resource to load, and loads it
    pub fn update(&mut self) {
        /// Secure access to the cache
        let cache = self.ptr.cache.upgrade();
        if let Some(cache) = cache {
            // the cache has not been dropped: request new version
            cache.get(&self.ptr.id).map(|new| *self = new );
        }
    }
}

impl<T> Deref for Cached<T>
{
    type Target = T;

    fn deref(&self) -> &T {
        self.ptr.inner.downcast_ref::<T>().unwrap()
    }
}*/

///
/// The problem with caches:
/// A cached object may contain references to things
/// thus it makes it non 'static
/// thus you can't use 'any' with it
///
/// Solution: all cached objects must be 'static
/// Use Rc for references, or weak refs
///
/// Cache design:
/// must implement the cacheobject trait
///
/// Cached objects live as long as the cache
pub struct Cache
{
    cached_objects: RefCell<HashMap<String, Box<CacheCell<Any>>>>,
    fs_watcher: RefCell<notify::RecommendedWatcher>,
    fs_events: Receiver<notify::DebouncedEvent>
}

impl ::std::fmt::Debug for Cache
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        Ok(())
    }
}

impl Cache
{
    pub fn new() -> Rc<Cache> {
        // setup notification channel
        let (tx,rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();

        Rc::new(Cache {
            cached_objects: RefCell::new(HashMap::new()),
            fs_events: rx,
            fs_watcher: RefCell::new(watcher)
        })
    }

    pub fn process_filesystem_events(&self)
    {
        // go through all filesystem events and see if one concerns an object in the cache
        for ev in self.fs_events.try_iter() {
            debug!("watch event: {:?}", ev);
        }
    }
}

pub enum ReloadReason
{
    Initial,
    FileCreated,
    FileModified,
    FileRemoved
}

pub trait CacheTrait
{
    fn add<T>(&self, path: String, obj: T) -> T where T: Any + Clone;
    fn get_or<T, F>(&self, path: &str, f: F) -> Option<T>
        where
            T: Any + Clone,
            F: FnOnce() -> T;
    fn get<T>(&self, path: &str) -> Option<T> where T: Any + Clone;

    fn add_and_watch<T, F>(&self, path: String, f: F) -> Option<T>
        where T: Any + Clone,
              F: Fn(&str, ReloadReason) -> Option<T>;

}


/// Proposition: the cache handles the file watching
/// add_and_watch(url, Fn(url, change) -> T) -> Cached<T>
///
/// Proposition: how about returning a value instead of an Rc?
/// Sometimes it's more convenient
/// The user can still wrap it into an Rc
/// Hot-reload: just query the cache again for an updated value
impl CacheTrait for Rc<Cache>
{
    /// replaces existing elements (does not invalidate previous versions,
    /// as they are allocated with Rc)
    /// returns a copy of the element
    fn add<T>(&self, path: String, obj: T) -> T
        where T: Any + Clone
    {
        let mut hash = self.cached_objects.borrow_mut();
        let newobj = Box::new( CacheCell::new( Rc::downgrade(self),path.to_owned(), obj.clone()));
        hash.insert(path, newobj);
        obj
    }

    fn add_and_watch<T, F>(&self, path: String, f: F) -> Option<T>
        where T: Any + Clone,
              F: Fn(&str, ReloadReason) -> Option<T>
    {
        let result = f(&path, ReloadReason::Initial).map(|val| self.add(path.clone(), val));
        // setup watch
        self.fs_watcher.borrow_mut().watch(&path, notify::RecursiveMode::NonRecursive);
        result
    }

    fn get_or<T, F>(&self, path: &str, f: F) -> Option<T>
        where
            T: Any + Clone,
            F: FnOnce() -> T
    {
        let mut hash = self.cached_objects.borrow_mut();
        // if the hashmap doesn't have an entry, call f(), box the returned value, add it to the hash,
        // downcast it to the concrete type and return it
        let obj = hash.entry(path.to_owned()).or_insert_with(|| {
            Box::new( CacheCell::new(Rc::downgrade(self),path.to_owned(), f().clone()))
        });

        obj.inner.downcast_ref::<T>().map(|v| v.clone())
    }

    fn get<T>(&self, path: &str) -> Option<T>
        where T: Any + Clone
    {
        // noice
        self.cached_objects.borrow().get(path).and_then(|obj| obj.inner.downcast_ref::<T>().map(|v| v.clone()))
    }
}
