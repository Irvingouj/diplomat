use crate::ffi;
use core::marker::PhantomData;

/// A slice of floats. `f64` is not `Eq`, so neither is `&[f64]`, and neither is a
/// struct that holds one even though every other derived trait holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatSliceView<'a> {
    pub values: &'a [f64],
    pub(crate) _lifetimes: PhantomData<*mut &'a ()>,
}
impl<'a> FloatSliceView<'a> {
    pub fn wrap(values: &'a [f64]) -> FloatSliceView<'a> {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        let result = unsafe { ffi::FloatSliceView_wrap(ffi::DiplomatSlice::from(values)) };
        FloatSliceView {
            values: result.values.into(),
            _lifetimes: core::marker::PhantomData,
        }
    }
    pub fn first(self) -> f64 {
        // SAFETY: generated arguments preserve the ownership, mutability, and lifetime constraints encoded by HIR.
        unsafe {
            ffi::FloatSliceView_first(ffi::FloatSliceView {
                values: ffi::DiplomatSlice::from(self.values),
            })
        }
    }
}
