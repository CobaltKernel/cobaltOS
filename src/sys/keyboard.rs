use pc_keyboard::{DecodedKey, KeyCode};
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

static LAST_KEY: Mutex<Option<DecodedKey>> = Mutex::new(None);

pub fn set_keycode(key_code: DecodedKey) {
    without_interrupts(|| {
        *LAST_KEY.lock() = Some(key_code)
    });
}

pub fn last_char() -> Option<char> {
    if let Some(key) = *LAST_KEY.lock() {
        match key {
            DecodedKey::RawKey(_) => return None,
            DecodedKey::Unicode(chr) => return Some(chr),
        }
    } else {
        return None;
    }
}

pub fn last_key() -> Option<KeyCode> {
    if let Some(key) = *LAST_KEY.lock() {
        match key {
            DecodedKey::RawKey(kc) => return Some(kc),
            DecodedKey::Unicode(_) => return None,
        }
    } else {
        return None;
    }
}

pub fn consume_char() -> Option<char> {
    let value = last_char();
    *LAST_KEY.lock() = None;
    return value;
}

pub fn consume_key() -> Option<KeyCode> {
    let value = last_key();
    *LAST_KEY.lock() = None;
    return value;
}