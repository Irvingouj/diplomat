use core::fmt;
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;

use crate::ffi;
#[allow(unused_imports)]
use crate::types::*;

pub struct OwnedSliceReturn {
    pub(crate) inner: NonNull<ffi::OwnedSliceReturn>,
    pub(crate) _not_send_sync: PhantomData<Rc<()>>,
}

pub struct OwnedSliceReturnRef<'view> {
    pub(crate) inner: NonNull<ffi::OwnedSliceReturn>,
    pub(crate) _borrow: PhantomData<&'view ()>,
    pub(crate) _not_send_sync: PhantomData<Rc<()>>,
}

pub struct OwnedSliceReturnRefMut<'view> {
    pub(crate) inner: NonNull<ffi::OwnedSliceReturn>,
    pub(crate) _borrow: PhantomData<&'view mut ()>,
    pub(crate) _not_send_sync: PhantomData<Rc<()>>,
}

#[doc(hidden)]
pub trait OwnedSliceReturnSharedArg: crate::private::OwnedSliceReturnSharedSealed {}

#[doc(hidden)]
pub trait OwnedSliceReturnMutArg: crate::private::OwnedSliceReturnMutSealed {}

impl crate::private::OwnedSliceReturnSharedSealed for OwnedSliceReturn {
    fn __as_const_ptr(&self) -> *const ffi::OwnedSliceReturn {
        self.inner.as_ptr()
    }
}

impl crate::private::OwnedSliceReturnMutSealed for OwnedSliceReturn {
    fn __as_mut_ptr(&mut self) -> *mut ffi::OwnedSliceReturn {
        self.inner.as_ptr()
    }
}

impl OwnedSliceReturnSharedArg for OwnedSliceReturn {}
impl OwnedSliceReturnMutArg for OwnedSliceReturn {}

impl<'view> crate::private::OwnedSliceReturnSharedSealed for OwnedSliceReturnRef<'view> {
    fn __as_const_ptr(&self) -> *const ffi::OwnedSliceReturn {
        self.inner.as_ptr()
    }
}

impl<'view> OwnedSliceReturnSharedArg for OwnedSliceReturnRef<'view> {}

impl<'view> crate::private::OwnedSliceReturnSharedSealed for OwnedSliceReturnRefMut<'view> {
    fn __as_const_ptr(&self) -> *const ffi::OwnedSliceReturn {
        self.inner.as_ptr()
    }
}

impl<'view> crate::private::OwnedSliceReturnMutSealed for OwnedSliceReturnRefMut<'view> {
    fn __as_mut_ptr(&mut self) -> *mut ffi::OwnedSliceReturn {
        self.inner.as_ptr()
    }
}

impl<'view> OwnedSliceReturnSharedArg for OwnedSliceReturnRefMut<'view> {}
impl<'view> OwnedSliceReturnMutArg for OwnedSliceReturnRefMut<'view> {}

impl Drop for OwnedSliceReturn {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the non-null handle and calls the provider destructor once.
        unsafe { ffi::OwnedSliceReturn_destroy(self.inner.as_ptr()) };
    }
}

impl fmt::Debug for OwnedSliceReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OwnedSliceReturn")
            .field(&self.inner.as_ptr())
            .finish()
    }
}

impl<'view> fmt::Debug for OwnedSliceReturnRef<'view> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OwnedSliceReturnRef")
            .field(&self.inner.as_ptr())
            .finish()
    }
}

impl<'view> fmt::Debug for OwnedSliceReturnRefMut<'view> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OwnedSliceReturnRefMut")
            .field(&self.inner.as_ptr())
            .finish()
    }
}

impl OwnedSliceReturn {
    /// Returns an owned `Box<[u8]>` of `len` bytes, each set to `(i % 256) as u8`.
    /// `len == 0` exercises the empty-buffer case; a large `len` exercises the
    /// GC memory-pressure path.
    pub fn make_bytes(len: u32) -> crate::DiplomatBoxU8 {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::OwnedSliceReturn_make_bytes(len) };
        crate::DiplomatBoxU8::from_abi(result)
    }
    /// Fallible variant of [Self::make_bytes]: errors on `len == 0`.
    /// Exercises the `Result<Box<[u8]>, E>` bridge shape.
    pub fn try_make_bytes(len: u32) -> Result<crate::DiplomatBoxU8, ErrorEnum> {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::OwnedSliceReturn_try_make_bytes(len) };
        match Result::from(result) {
            Ok(result) => Ok(crate::DiplomatBoxU8::from_abi(result)),
            Err(result) => Err(result),
        }
    }
    /// How many heap allocations this provider library still holds.
    /// Dropping an owned byte buffer must bring this back down. The copy a
    /// caller makes with `clone_to_box` is allocated on their side and is
    /// not included.
    pub fn provider_live_allocations() -> usize {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        unsafe { ffi::OwnedSliceReturn_provider_live_allocations() }
    }
}
