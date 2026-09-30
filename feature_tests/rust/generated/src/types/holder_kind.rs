use crate::ffi;

/// The same shape on an enum, whose generated file carries even fewer imports.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HolderKind {
    First = 0,
    Second = 1,
}
impl HolderKind {
    pub fn make() -> crate::Opaque {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::HolderKind_make() };
        {
            let inner = core::ptr::NonNull::new(result as *mut _)
                .expect("Diplomat ABI returned null for non-null Opaque");
            crate::Opaque {
                inner,
                _not_send_sync: core::marker::PhantomData,
            }
        }
    }
}
