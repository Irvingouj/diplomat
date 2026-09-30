use crate::ffi;
use core::marker::PhantomData;

/// A method on a lifetime-carrying struct. The receiver is the one place that names
/// the struct's lifetime outside the struct's own declaration, and the `extern`
/// block that declares the method does not declare the lifetime: it names a
/// placeholder, because the ABI does not carry one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorrowedBytes<'a> {
    pub bytes: &'a [u8],
    pub(crate) _lifetimes: PhantomData<*mut &'a ()>,
}
impl<'a> BorrowedBytes<'a> {
    /// The caller's own bytes, borrowed for as long as the returned value lives.
    pub fn from_bytes(bytes: &'a [u8]) -> BorrowedBytes<'a> {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::BorrowedBytes_from_bytes(ffi::DiplomatSlice::from(bytes)) };
        BorrowedBytes {
            bytes: result.bytes.into(),
            _lifetimes: core::marker::PhantomData,
        }
    }
    pub fn len(self) -> usize {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        unsafe {
            ffi::BorrowedBytes_len(ffi::BorrowedBytes {
                bytes: ffi::DiplomatSlice::from(self.bytes),
            })
        }
    }
}
