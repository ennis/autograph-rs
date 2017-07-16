use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::marker::PhantomData;
use std::ops::Deref;

pub struct CacheObject
{
}



///
/// Wrapper around a cached object of type T
#[derive(Clone)]
pub struct Cached<T: 'static>
{
    ptr: Rc<Any>,
    _phantom: PhantomData<T>
}

impl<T> Cached<T>
{
    fn new(value: T) -> Cached<T> {
        unsafe {
            Self::from_rc_any(Rc::new(value)).unwrap()
        }
    }

    unsafe fn from_rc_any(ptr: Rc<Any>) -> Option<Cached<T>> {
        match ptr.downcast_ref::<T>() {
            Some(_) => Some(Cached { ptr, _phantom: PhantomData }),
            None => None
        }
    }
}

impl<T> Deref for Cached<T>
{
    type Target = T;

    fn deref(&self) -> &T {
        self.ptr.downcast_ref::<T>().unwrap()
    }
}

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
    cached_objects: RefCell<HashMap<String, Rc<Any>>>
}

impl Cache
{
    /// panics if element already there
    pub fn add<T>(&self, path: String, obj: T) -> Cached<T>
        where T: Any
    {
        let mut hash = self.cached_objects.borrow_mut();
        let mut obj = match hash.entry(path) {
            Entry::Occupied(_) => { panic!("Object already present in cache") },
            Entry::Vacant(v) => { v.insert(Rc::new(obj)) }
        }.clone();

        unsafe {
            Cached::from_rc_any(obj).unwrap()
        }
    }

    pub fn get_or<'cache, T, F>(&'cache self, path: &str, f: F) -> Option<Cached<T>>
        where
            T: Any,
            F: FnOnce() -> T
    {
        let mut hash = self.cached_objects.borrow_mut();
        // if the hashmap doesn't have an entry, call f(), box the returned value, add it to the hash,
        // downcast it to the concrete type and return it
        let obj = hash.entry(path.to_owned()).or_insert_with(|| { Rc::new(f()) }).clone();

        unsafe {
            Cached::from_rc_any(obj)
        }
    }

    pub fn get<'cache, T>(&'cache self, path: &str) -> Option<Cached<T>>
    {
        unimplemented!()
    }
}