//! A reimplementation of Any without the 'static bound
//! All trait bounds are lost

extern "rust-intrinsic" {
    // redeclare the type_id intrinsic (while no one is looking...) to remove the 'static bound
    // hopefully no one sees this until the relevant RFC is implemented:
    // https://github.com/rust-lang/rfcs/blob/master/text/1849-non-static-type-id.md
    // TODO find another solution before publishing
    // TODO I don't want to put Rc's everywhere
    fn type_id<T: ?Sized>() -> u64;
}

#[derive(Eq,PartialEq,Copy,Clone,Debug)]
struct RawTypeId(u64);

fn type_id_of_unbound<T: ?Sized>() -> RawTypeId
{
    unsafe {
        RawTypeId(type_id::<T>())
    }
}

pub trait UnsafeAny
{
    fn get_type_id(&self) -> RawTypeId;
}

impl<T: ?Sized> UnsafeAny for T
{
    fn get_type_id(&self) -> RawTypeId
    {
        unsafe {
            type_id_of_unbound::<T>()
        }
    }
}

impl UnsafeAny {
    pub unsafe fn is<T: UnsafeAny>(&self) -> bool {
        // Get TypeId of the type this function is instantiated with
        let t = type_id_of_unbound::<T>();

        // Get TypeId of the type in the trait object
        let boxed = self.get_type_id();

        // Compare both TypeIds on equality
        t == boxed
    }

    pub unsafe fn downcast_ref<T: UnsafeAny>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                Some(&*(self as *const UnsafeAny as *const T))
            }
        } else {
            None
        }
    }

    pub unsafe fn downcast_mut<T: UnsafeAny>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe {
                Some(&mut *(self as *mut UnsafeAny as *mut T))
            }
        } else {
            None
        }
    }
}
