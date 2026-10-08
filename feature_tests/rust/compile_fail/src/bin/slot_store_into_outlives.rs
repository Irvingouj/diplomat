#![forbid(unsafe_code)]

//! `Slot::store_into` must keep the lifetime that ties `value` to `slot`.
use diplomat_rust_backend_generated::Slot;

fn main() {
    let mut slot: Slot<'static> = Slot::new(b"initial");
    {
        let temporary = vec![42_u8; 8];
        Slot::store_into(&mut slot, &temporary, b"long", b"short");
    }
    let dangling: &'static [u8] = slot.get();
    let _ = dangling[0];
}
