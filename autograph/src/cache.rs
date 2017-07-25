use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use std::cell::RefCell;
use std::mem;
use unsafe_any::UnsafeAny;

pub struct CacheObject
{

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
    cached_objects: RefCell<HashMap<String, Box<UnsafeAny>>>
}

impl Cache
{
    /// panics if element already there
    /*pub fn add<'cache, T>(&'cache self, path: String, obj: T) -> &'cache T
        where T: Any + 'static
    {
        let mut hash = self.cached_objects.borrow_mut();
        let mut obj = match hash.entry(path) {
            Entry::Occupied(_) => { panic!("Object already present in cache") },
            Entry::Vacant(v) => { v.insert(Box::new(obj)) }
        }.downcast_ref::<T>().unwrap();
        // see get_or for an explanation
        unsafe {
            mem::transmute(obj)
        }
    }*/

    // The trait bounds on T should only be 'cache, but it still needs 'static because of Any
    // TODO is there an UnsafeAny?
    pub fn get_or<'cache, T, F>(&'cache self, path: &str, f: F) -> &'cache T
        where T: 'cache,
              F: FnOnce() -> T
    {
        let mut hash = self.cached_objects.borrow_mut();
        // if the hashmap doesn't have an entry, call f(), box the returned value, add it to the hash,
        // downcast it to the concrete type and return it
        let obj = unsafe {
            // So, we first box the object, then cast it into a boxed trait object
            // We then transmute it to remove the lifetime bound of the UnsafeAny
            // Legal disclaimer: this is highly unsafe, etc.
            hash.entry(path.to_owned()).or_insert_with(|| { mem::transmute(Box::new(f()) as Box<UnsafeAny>) }).downcast_ref::<T>().unwrap()
        };

        unsafe {
            // Technically, obj is only valid for the duration of the dynamic borrow_mut()
            // However, since an user cannot remove an object from the cache other than by dropping
            // the cache in its entirety, and since the cached objects are stored inside boxes,
            // adding an element to the hash map won't invalidate the existing references.
            // Thus, we can safely extend the lifetime of the reference to the cached object
            // to the lifetime of the 'cache itself
            mem::transmute(obj)
        }
    }

    pub fn get<'cache, T>(&'cache self, path: &str) -> Option<&'cache T>
    {
        unimplemented!()
    }
}