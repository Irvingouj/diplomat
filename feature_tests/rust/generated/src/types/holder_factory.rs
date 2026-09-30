use crate::ffi;

/// A value type whose method hands out an opaque it owns.
///
/// The generated file for a value type is not the opaque's module: it has its own
/// imports, so anything the shared method renderer spells unqualified has to be
/// resolvable there too. Rust-only: the other backends are not part of this
/// regression, and their generated output must not move for it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HolderFactory {
    pub tag: u8,
}
impl HolderFactory {
    pub fn make() -> crate::Opaque {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::HolderFactory_make() };
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
