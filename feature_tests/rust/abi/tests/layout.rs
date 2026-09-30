//! The transcription against the runtime: same size, same alignment, and the data
//! lands where the transcription says the fields are.
//!
//! Sizes and alignments are compared directly. Offsets are not: the runtime's fields
//! are private, so the only honest way to ask where the runtime puts a field is to
//! have the runtime build a value and look at its bytes.

use core::mem::{align_of, size_of};

use diplomat_runtime as runtime;
use diplomat_rust_backend_generated::abi;

/// The transcription's layout for one type, by name.
fn ours(name: &str) -> abi::layout::Layout {
    *abi::layout::LAYOUTS
        .iter()
        .find(|layout| layout.name == name)
        .unwrap_or_else(|| panic!("the transcription records no layout for {name}"))
}

/// The offset the transcription claims for one of a type's fields.
fn offset_of_field(name: &str, field: &str) -> usize {
    ours(name)
        .fields
        .iter()
        .find(|(candidate, _)| *candidate == field)
        .map(|(_, offset)| *offset)
        .unwrap_or_else(|| panic!("the transcription records no {field} in {name}"))
}

/// The bytes of a value, for reading where its own constructor put its fields.
fn bytes_of<T>(value: &T) -> &[u8] {
    // SAFETY: every value read here is a `repr(C)` struct of integers and raw
    // pointers. This only looks at the bytes; it never builds a reference to a field.
    unsafe { core::slice::from_raw_parts(value as *const T as *const u8, size_of::<T>()) }
}

/// The `usize` at `offset`, as the ABI lays one out.
fn usize_at(raw: &[u8], offset: usize) -> usize {
    let mut word = [0u8; size_of::<usize>()];
    word.copy_from_slice(&raw[offset..offset + size_of::<usize>()]);
    usize::from_ne_bytes(word)
}

/// The `u64` at `offset`, as the ABI lays one out.
fn u64_at(raw: &[u8], offset: usize) -> u64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(&raw[offset..offset + 8]);
    u64::from_ne_bytes(word)
}

#[test]
fn a_shipped_slice_is_a_pointer_then_a_length() {
    let transcription = ours("DiplomatSlice");
    assert_eq!(
        (transcription.size, transcription.align),
        (
            size_of::<runtime::DiplomatSlice<'static, u8>>(),
            align_of::<runtime::DiplomatSlice<'static, u8>>()
        ),
        "DiplomatSlice disagrees with the runtime"
    );

    let data = [1u8, 2, 3];
    let shipped = runtime::DiplomatSlice::from(&data[..]);
    let raw = bytes_of(&shipped);
    assert_eq!(
        usize_at(raw, offset_of_field("DiplomatSlice", "ptr")),
        data.as_ptr() as usize,
        "the runtime does not put the pointer where the transcription says"
    );
    assert_eq!(
        usize_at(raw, offset_of_field("DiplomatSlice", "len")),
        data.len(),
        "the runtime does not put the length where the transcription says"
    );
}

#[test]
fn a_shipped_mutable_slice_is_a_pointer_then_a_length() {
    let transcription = ours("DiplomatSliceMut");
    assert_eq!(
        (transcription.size, transcription.align),
        (
            size_of::<runtime::DiplomatSliceMut<'static, u8>>(),
            align_of::<runtime::DiplomatSliceMut<'static, u8>>()
        ),
        "DiplomatSliceMut disagrees with the runtime"
    );

    let mut data = [4u8, 5, 6, 7];
    let expected_ptr = data.as_ptr() as usize;
    let shipped = runtime::DiplomatSliceMut::from(&mut data[..]);
    let raw = bytes_of(&shipped);
    assert_eq!(
        usize_at(raw, offset_of_field("DiplomatSliceMut", "ptr")),
        expected_ptr
    );
    assert_eq!(usize_at(raw, offset_of_field("DiplomatSliceMut", "len")), 4);
}

#[test]
fn a_shipped_owned_slice_is_a_pointer_then_a_length() {
    let transcription = ours("DiplomatOwnedSlice");
    assert_eq!(
        (transcription.size, transcription.align),
        (
            size_of::<runtime::DiplomatOwnedSlice<u8>>(),
            align_of::<runtime::DiplomatOwnedSlice<u8>>()
        ),
        "DiplomatOwnedSlice disagrees with the runtime"
    );

    let owned = runtime::DiplomatOwnedSlice::from(vec![8u8, 9].into_boxed_slice());
    let expected_ptr = owned.as_ptr() as usize;
    let raw = bytes_of(&owned);
    assert_eq!(
        usize_at(raw, offset_of_field("DiplomatOwnedSlice", "ptr")),
        expected_ptr
    );
    assert_eq!(
        usize_at(raw, offset_of_field("DiplomatOwnedSlice", "len")),
        2
    );
    // `owned` frees the buffer as it goes out of scope, through the runtime's own
    // destructor. That is the piece of behaviour the transcription deliberately does
    // not copy: the consumer frees the provider's buffer by calling the provider's
    // exported destructor instead.
}

#[test]
fn a_shipped_result_is_a_payload_then_a_tag() {
    let transcription = ours("DiplomatResult");
    assert_eq!(
        (transcription.size, transcription.align),
        (
            size_of::<runtime::DiplomatResult<u32, u64>>(),
            align_of::<runtime::DiplomatResult<u32, u64>>()
        ),
        "DiplomatResult disagrees with the runtime"
    );

    let payload = offset_of_field("DiplomatResult", "value");
    let tag = offset_of_field("DiplomatResult", "is_ok");

    let ok = runtime::DiplomatResult::<u32, u64>::from(Ok(0x1234_5678));
    let raw = bytes_of(&ok);
    assert_eq!(
        u64_at(raw, payload) as u32,
        0x1234_5678,
        "the runtime does not put the payload where the transcription says"
    );
    assert_eq!(
        raw[tag], 1,
        "the runtime does not put the ok tag where the transcription says"
    );

    let err = runtime::DiplomatResult::<u32, u64>::from(Err(u64::MAX));
    let raw = bytes_of(&err);
    assert_eq!(u64_at(raw, payload), u64::MAX);
    assert_eq!(
        raw[tag], 0,
        "the runtime does not put the err tag where the transcription says"
    );
}

/// The payload types the generated code instantiates actually use: same size and
/// alignment on both sides, whatever the payload's own alignment is.
#[test]
fn every_instantiation_the_generated_code_names_agrees_on_size_and_alignment() {
    fn same<Ours, Theirs>(what: &str) {
        assert_eq!(size_of::<Ours>(), size_of::<Theirs>(), "{what}: size");
        assert_eq!(align_of::<Ours>(), align_of::<Theirs>(), "{what}: align");
    }

    same::<abi::DiplomatResult<u8, u8>, runtime::DiplomatResult<u8, u8>>("DiplomatResult<u8, u8>");
    same::<abi::DiplomatResult<u32, u64>, runtime::DiplomatResult<u32, u64>>(
        "DiplomatResult<u32, u64>",
    );
    same::<abi::DiplomatOption<f64>, runtime::DiplomatOption<f64>>("DiplomatOption<f64>");
    same::<
        abi::DiplomatResult<abi::DiplomatSlice<'static, u8>, ()>,
        runtime::DiplomatResult<runtime::DiplomatSlice<'static, u8>, ()>,
    >("DiplomatResult<DiplomatSlice, ()>");
    same::<
        abi::DiplomatOption<abi::DiplomatOwnedSlice<u8>>,
        runtime::DiplomatOption<runtime::DiplomatOwnedSlice<u8>>,
    >("DiplomatOption<DiplomatOwnedSlice>");
    same::<abi::DiplomatSlice<'static, f64>, runtime::DiplomatSlice<'static, f64>>(
        "DiplomatSlice<f64>",
    );
    same::<abi::DiplomatSliceMut<'static, f64>, runtime::DiplomatSliceMut<'static, f64>>(
        "DiplomatSliceMut<f64>",
    );
}
