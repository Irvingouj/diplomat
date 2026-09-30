use crate::ffi;
use core::marker::PhantomData;

/// A borrowed *mutable* slice field. `&mut [T]` is `PartialEq` and `Eq`, but it is
/// neither `Clone` nor `Copy`, so a struct holding one cannot derive those.
#[derive(Debug, PartialEq, Eq)]
pub struct MutableBorrowedBytes<'a> {
    pub bytes: &'a mut [u8],
    pub(crate) _lifetimes: PhantomData<*mut &'a ()>,
}
impl<'a> MutableBorrowedBytes<'a> {
    pub fn wrap(bytes: &'a mut [u8]) -> MutableBorrowedBytes<'a> {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::MutableBorrowedBytes_wrap(ffi::DiplomatSliceMut::from(bytes)) };
        MutableBorrowedBytes {
            bytes: result.bytes.into(),
            _lifetimes: core::marker::PhantomData,
        }
    }
    pub fn write_first(self, value: u8) -> u8 {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        unsafe {
            ffi::MutableBorrowedBytes_write_first(
                ffi::MutableBorrowedBytes {
                    bytes: ffi::DiplomatSliceMut::from(self.bytes),
                },
                value,
            )
        }
    }
}
