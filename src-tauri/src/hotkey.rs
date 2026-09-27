use anyhow::{anyhow, bail, Context, Result};
use evdev::{Device, EventSummary, EventType, KeyCode};
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

/// A parsed hotkey: a set of modifier keys plus one trigger key.
struct ParsedHotkey {
    modifiers: Vec<KeyCode>,
    trigger: KeyCode,
}

/// Parses strings like "Ctrl+Alt+Space", "Shift+F9", or just "F9".
fn parse_hotkey(s: &str) -> Result<ParsedHotkey> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() || parts.last().map_or(true, |p| p.is_empty()) {
        bail!("empty hotkey string");
    }

    let mut modifiers = Vec::new();
    for part in &parts[..parts.len() - 1] {
        let key = match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => KeyCode::KEY_LEFTCTRL,
            "ALT" => KeyCode::KEY_LEFTALT,
            "SHIFT" => KeyCode::KEY_LEFTSHIFT,
            "SUPER" | "META" | "WIN" => KeyCode::KEY_LEFTMETA,
            other => bail!("unknown modifier '{other}' (supported: Ctrl, Alt, Shift, Super)"),
        };
        modifiers.push(key);
    }

    let trigger_str = parts.last().unwrap();
    let trigger =
        parse_key_name(trigger_str).ok_or_else(|| anyhow!("unknown key '{trigger_str}'"))?;

    Ok(ParsedHotkey { modifiers, trigger })
}

/// Maps a plain key name to its evdev KeyCode constant.
fn parse_key_name(s: &str) -> Option<KeyCode> {
    let upper = s.to_uppercase();
    let key = match upper.as_str() {
        "A" => KeyCode::KEY_A,
        "B" => KeyCode::KEY_B,
        "C" => KeyCode::KEY_C,
        "D" => KeyCode::KEY_D,
        "E" => KeyCode::KEY_E,
        "F" => KeyCode::KEY_F,
        "G" => KeyCode::KEY_G,
        "H" => KeyCode::KEY_H,
        "I" => KeyCode::KEY_I,
        "J" => KeyCode::KEY_J,
        "K" => KeyCode::KEY_K,
        "L" => KeyCode::KEY_L,
        "M" => KeyCode::KEY_M,
        "N" => KeyCode::KEY_N,
        "O" => KeyCode::KEY_O,
        "P" => KeyCode::KEY_P,
        "Q" => KeyCode::KEY_Q,
        "R" => KeyCode::KEY_R,
        "S" => KeyCode::KEY_S,
        "T" => KeyCode::KEY_T,
        "U" => KeyCode::KEY_U,
        "V" => KeyCode::KEY_V,
        "W" => KeyCode::KEY_W,
        "X" => KeyCode::KEY_X,
        "Y" => KeyCode::KEY_Y,
        "Z" => KeyCode::KEY_Z,
        "0" => KeyCode::KEY_0,
        "1" => KeyCode::KEY_1,
        "2" => KeyCode::KEY_2,
        "3" => KeyCode::KEY_3,
        "4" => KeyCode::KEY_4,
        "5" => KeyCode::KEY_5,
        "6" => KeyCode::KEY_6,
        "7" => KeyCode::KEY_7,
        "8" => KeyCode::KEY_8,
        "9" => KeyCode::KEY_9,
        "F1" => KeyCode::KEY_F1,
        "F2" => KeyCode::KEY_F2,
        "F3" => KeyCode::KEY_F3,
        "F4" => KeyCode::KEY_F4,
        "F5" => KeyCode::KEY_F5,
        "F6" => KeyCode::KEY_F6,
        "F7" => KeyCode::KEY_F7,
        "F8" => KeyCode::KEY_F8,
        "F9" => KeyCode::KEY_F9,
        "F10" => KeyCode::KEY_F10,
        "F11" => KeyCode::KEY_F11,
        "F12" => KeyCode::KEY_F12,
        "SPACE" => KeyCode::KEY_SPACE,
        "TAB" => KeyCode::KEY_TAB,
        "ENTER" | "RETURN" => KeyCode::KEY_ENTER,
        "ESCAPE" | "ESC" => KeyCode::KEY_ESC,
        "BACKSPACE" => KeyCode::KEY_BACKSPACE,
        "INSERT" => KeyCode::KEY_INSERT,
        "DELETE" => KeyCode::KEY_DELETE,
        "HOME" => KeyCode::KEY_HOME,
        "END" => KeyCode::KEY_END,
        "PAGEUP" => KeyCode::KEY_PAGEUP,
        "PAGEDOWN" => KeyCode::KEY_PAGEDOWN,
        "UP" => KeyCode::KEY_UP,
        "DOWN" => KeyCode::KEY_DOWN,
        "LEFT" => KeyCode::KEY_LEFT,
        "RIGHT" => KeyCode::KEY_RIGHT,
        "SCROLLLOCK" => KeyCode::KEY_SCROLLLOCK,
        "PAUSE" => KeyCode::KEY_PAUSE,
        _ => return None,
    };
    Some(key)
}

/// Checks whether a modifier requirement is satisfied, treating left/right
/// physical variants as equivalent.
fn modifier_pressed(modifier: KeyCode, held: &HashSet<KeyCode>) -> bool {
    match modifier {
        KeyCode::KEY_LEFTCTRL => {
            held.contains(&KeyCode::KEY_LEFTCTRL) || held.contains(&KeyCode::KEY_RIGHTCTRL)
        }
        KeyCode::KEY_LEFTALT => {
            held.contains(&KeyCode::KEY_LEFTALT) || held.contains(&KeyCode::KEY_RIGHTALT)
        }
        KeyCode::KEY_LEFTSHIFT => {
            held.contains(&KeyCode::KEY_LEFTSHIFT) || held.contains(&KeyCode::KEY_RIGHTSHIFT)
        }
        KeyCode::KEY_LEFTMETA => {
            held.contains(&KeyCode::KEY_LEFTMETA) || held.contains(&KeyCode::KEY_RIGHTMETA)
        }
        other => held.contains(&other),
    }
}

/// Finds all keyboard-capable input devices.
fn find_keyboards() -> Result<Vec<Device>> {
    let mut keyboards = Vec::new();
    for (_path, device) in evdev::enumerate() {
        let supports_keys = device.supported_events().contains(EventType::KEY);
        let has_letter_keys = device
            .supported_keys()
            .map_or(false, |keys| keys.contains(KeyCode::KEY_A));

        if supports_keys && has_letter_keys {
            keyboards.push(device);
        }
    }

    if keyboards.is_empty() {
        bail!("no keyboard devices found — check you're in the `input` group and relogged");
    }

    Ok(keyboards)
}

pub struct PushToTalk {
    receiver: Receiver<HotkeyEvent>,
}

impl PushToTalk {
    pub fn new(combo: &str) -> Result<Self> {
        let parsed = parse_hotkey(combo).context("invalid hotkey in config")?;
        let keyboards = find_keyboards()?;

        let (tx, rx) = mpsc::channel();

        for mut device in keyboards {
            let tx = tx.clone();
            let modifiers = parsed.modifiers.clone();
            let trigger = parsed.trigger;

            std::thread::spawn(move || {
                let mut held: HashSet<KeyCode> = HashSet::new();
                let mut combo_active = false;

                loop {
                    let events = match device.fetch_events() {
                        Ok(events) => events,
                        Err(_) => break,
                    };

                    for event in events {
                        if let EventSummary::Key(_, key, value) = event.destructure() {
                            match value {
                                1 => {
                                    held.insert(key);
                                }
                                0 => {
                                    held.remove(&key);
                                }
                                _ => continue, // 2 = autorepeat
                            }

                            let all_modifiers_held =
                                modifiers.iter().all(|m| modifier_pressed(*m, &held));
                            let trigger_held = held.contains(&trigger);
                            let now_active = all_modifiers_held && trigger_held;

                            if now_active && !combo_active {
                                combo_active = true;
                                let _ = tx.send(HotkeyEvent::Pressed);
                            } else if !now_active && combo_active {
                                combo_active = false;
                                let _ = tx.send(HotkeyEvent::Released);
                            }
                        }
                    }
                }
            });
        }

        Ok(PushToTalk { receiver: rx })
    }

    pub fn wait_for_press(&self) {
        loop {
            if let Ok(HotkeyEvent::Pressed) = self.receiver.recv_timeout(Duration::from_secs(3600))
            {
                return;
            }
        }
    }

    pub fn wait_for_release(&self) {
        loop {
            if let Ok(HotkeyEvent::Released) = self.receiver.recv_timeout(Duration::from_secs(3600))
            {
                return;
            }
        }
    }
}
