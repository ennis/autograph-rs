
use std::marker::PhantomData;

pub struct Frame<'ctx>
{
    _phantom: PhantomData<&'ctx ()>
}
