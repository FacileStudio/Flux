//! The key buffer checks: what it keeps, what it drops, and that it never
//! moves the text it holds.

use gtk4::gdk::{Key, ModifierType};

use super::*;

fn buffer_of(text: &str) -> KeyBuffer {
    let mut buffer = KeyBuffer::new();
    for character in text.chars() {
        buffer.press(Typed::Character(character));
    }
    buffer
}

#[test]
fn keys_are_held_in_order_and_handed_over_once() {
    let mut buffer = buffer_of("s3cr\u{e9}t");
    assert_eq!(buffer.take().as_str(), "s3cr\u{e9}t");
    assert_eq!(buffer.take().as_str(), "", "taking leaves nothing behind");
}

#[test]
fn backspace_erases_one_character_and_escape_all_of_them() {
    let mut buffer = buffer_of("ab\u{e9}");
    buffer.press(Typed::Erase);
    buffer.press(Typed::Nothing);
    assert_eq!(buffer.typed.as_str(), "ab");
    buffer.press(Typed::Erase);
    buffer.press(Typed::Erase);
    buffer.press(Typed::Erase);
    assert_eq!(buffer.typed.as_str(), "", "erasing an empty buffer is harmless");

    let mut buffer = buffer_of("abc");
    buffer.press(Typed::Forget);
    assert_eq!(buffer.take().as_str(), "");
}

#[test]
fn the_buffer_never_grows_so_the_text_is_never_copied() {
    let mut buffer = KeyBuffer::new();
    let start = buffer.typed.as_ptr();
    for _ in 0..CAPACITY * 4 {
        buffer.press(Typed::Character('x'));
    }
    assert_eq!(buffer.typed.len(), CAPACITY);
    assert_eq!(buffer.typed.capacity(), CAPACITY);
    assert_eq!(buffer.typed.as_ptr(), start, "the allocation did not move");

    buffer.press(Typed::Erase);
    buffer.press(Typed::Character('\u{e9}'));
    assert_eq!(buffer.typed.len(), CAPACITY - 1, "a character that does not fit is dropped whole");
    assert_eq!(buffer.typed.as_ptr(), start);

    buffer.press(Typed::Forget);
    assert_eq!(buffer.typed.as_ptr(), start, "forgetting wipes in place");
    assert_eq!(buffer.typed.capacity(), CAPACITY);
}

#[test]
fn forgetting_zeroes_the_bytes_that_held_the_text() {
    let mut buffer = buffer_of("hunter2");
    let start = buffer.typed.as_ptr();
    buffer.press(Typed::Forget);
    let wiped = unsafe { std::slice::from_raw_parts(start, CAPACITY) };
    assert!(wiped.iter().all(|byte| *byte == 0));
}

#[test]
fn only_plain_characters_type() {
    let none = ModifierType::empty();
    assert_eq!(typed(Key::a, none), Typed::Character('a'));
    assert_eq!(typed(Key::A, ModifierType::SHIFT_MASK), Typed::Character('A'));
    assert_eq!(typed(Key::eacute, ModifierType::LOCK_MASK), Typed::Character('\u{e9}'));
    assert_eq!(typed(Key::space, none), Typed::Character(' '));
    assert_eq!(typed(Key::BackSpace, none), Typed::Erase);
    assert_eq!(typed(Key::Escape, none), Typed::Forget);
    assert_eq!(typed(Key::Escape, ModifierType::CONTROL_MASK), Typed::Forget);
    assert_eq!(typed(Key::Return, none), Typed::Nothing, "Enter is never replayed");
    assert_eq!(typed(Key::KP_Enter, none), Typed::Nothing);
    assert_eq!(typed(Key::Tab, none), Typed::Nothing);
    assert_eq!(typed(Key::Shift_L, none), Typed::Nothing);
    assert_eq!(typed(Key::dead_circumflex, none), Typed::Nothing);
    assert_eq!(typed(Key::a, ModifierType::CONTROL_MASK), Typed::Nothing);
    assert_eq!(typed(Key::l, ModifierType::SUPER_MASK), Typed::Nothing);
    assert_eq!(typed(Key::BackSpace, ModifierType::ALT_MASK), Typed::Nothing);
}
