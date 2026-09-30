//! The ABI container hands each payload over exactly once.
//!
//! `DiplomatResult` holds a payload in a union and drops whichever arm the tag
//! selects. Moving a payload out therefore has to suppress that destructor, and a
//! read that copies the union field instead moves it is a payload freed twice —
//! harmless for an integer, a double free for anything owning memory. These tests
//! count drops rather than trust the allocator to notice.

use core::cell::Cell;
use std::rc::Rc;

use diplomat_rust_backend_generated::abi::{DiplomatOption, DiplomatResult};

/// A payload that counts how many times it is dropped.
struct Counted(Rc<Cell<usize>>);

impl Drop for Counted {
    fn drop(&mut self) {
        let count = self.0.get();
        self.0.set(count + 1);
    }
}

#[test]
fn the_ok_payload_of_a_converted_result_is_dropped_once() {
    let drops = Rc::new(Cell::new(0));

    let shipped = DiplomatResult::<Counted, ()>::from(Ok(Counted(drops.clone())));
    let converted: Result<Counted, ()> = shipped.into();
    assert_eq!(
        drops.get(),
        0,
        "converting must hand the payload over, not drop it"
    );

    drop(converted.expect("the ok arm is live"));
    assert_eq!(drops.get(), 1, "the payload must be dropped exactly once");
}

#[test]
fn the_err_payload_of_a_converted_result_is_dropped_once() {
    let drops = Rc::new(Cell::new(0));

    let shipped = DiplomatResult::<(), Counted>::from(Err(Counted(drops.clone())));
    let converted: Result<(), Counted> = shipped.into();
    assert_eq!(drops.get(), 0);

    drop(converted.expect_err("the err arm is live"));
    assert_eq!(drops.get(), 1);
}

#[test]
fn an_unconverted_result_drops_its_live_payload_once() {
    let drops = Rc::new(Cell::new(0));

    let shipped = DiplomatResult::<Counted, ()>::from(Ok(Counted(drops.clone())));
    drop(shipped);
    assert_eq!(drops.get(), 1);

    let drops = Rc::new(Cell::new(0));
    let shipped = DiplomatResult::<(), Counted>::from(Err(Counted(drops.clone())));
    drop(shipped);
    assert_eq!(drops.get(), 1);
}

#[test]
fn an_option_round_trips_without_dropping_its_payload() {
    let drops = Rc::new(Cell::new(0));

    let some: DiplomatOption<Counted> = DiplomatOption::from(Some(Counted(drops.clone())));
    let back: Option<Counted> = some.into();
    assert_eq!(drops.get(), 0);
    drop(back);
    assert_eq!(drops.get(), 1);

    let none: DiplomatOption<Counted> = DiplomatOption::from(None);
    assert!(none.into_option().is_none());
    assert_eq!(drops.get(), 1);
}

/// The payload that made the difference visible in the first place: an owned byte
/// buffer from the provider, converted out of a `Result` and then used.
#[test]
fn an_owned_byte_payload_survives_the_round_trip() {
    let payload: Box<[u8]> = vec![7u8; 32].into_boxed_slice();
    let shipped = DiplomatResult::<Box<[u8]>, ()>::from(Ok(payload));
    let converted: Result<Box<[u8]>, ()> = shipped.into();

    let payload = converted.expect("the ok arm is live");
    assert_eq!(payload.len(), 32);
    assert!(payload.iter().all(|byte| *byte == 7));

    let shipped = DiplomatResult::<(), Box<[u8]>>::from(Err(vec![1u8, 2].into_boxed_slice()));
    let converted: Result<(), Box<[u8]>> = shipped.into();
    assert_eq!(converted.expect_err("the err arm is live").len(), 2);
}
