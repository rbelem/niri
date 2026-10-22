//! Parsing of XDG shortcuts-spec trigger strings.
//!
//! The XDG shortcuts specification writes a shortcut trigger as a
//! `+`-separated list of modifier names followed by a keysym name, for example
//! `CTRL+ALT+T`. The GlobalShortcuts portal passes such strings to the
//! compositor in the `preferred_trigger` option, so niri has to understand
//! them.

use smithay::input::keyboard::keysyms::KEY_NoSymbol;
use smithay::input::keyboard::xkb::{keysym_from_name, keysym_get_name, KEYSYM_CASE_INSENSITIVE};
use smithay::input::keyboard::Keysym;

use crate::binds::{Key, Modifiers, Trigger};

/// Parses a trigger in the XDG shortcuts-spec format.
///
/// The modifier names are `CTRL`, `ALT`, `SHIFT`, `LOGO` (also spelled
/// `SUPER`) and `NUM`. The last part is a keysym name such as `T`, `space` or
/// `KP_1`. Names are matched case-insensitively.
///
/// Returns `None` when the trigger has no keysym part or when the keysym is
/// unknown.
pub fn parse_shortcuts_spec_trigger(trigger: &str) -> Option<Key> {
    let mut modifiers = Modifiers::empty();
    let mut keysym_name = None;

    for part in trigger.split('+') {
        let part = part.trim();
        match part.to_ascii_uppercase().as_str() {
            "CTRL" => modifiers |= Modifiers::CTRL,
            "ALT" => modifiers |= Modifiers::ALT,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "LOGO" | "SUPER" => modifiers |= Modifiers::SUPER,
            // The numeric lock modifier has no equivalent in niri.
            "NUM" => (),
            "" => return None,
            _ => {
                // The keysym must be the last part of the trigger.
                if keysym_name.is_some() {
                    return None;
                }
                keysym_name = Some(part);
            }
        }
    }

    let keysym = keysym_from_name(keysym_name?, KEYSYM_CASE_INSENSITIVE);
    if keysym.raw() == KEY_NoSymbol {
        return None;
    }

    Some(Key {
        trigger: Trigger::Keysym(keysym),
        modifiers,
    })
}

/// Returns a human-readable rendering of a key combination.
///
/// The modifiers come first, in a fixed order, followed by the keysym name.
pub fn key_description(key: &Key) -> String {
    let Trigger::Keysym(keysym) = key.trigger else {
        return String::from("unsupported trigger");
    };

    let mut description = String::new();
    for (modifier, name) in [
        (Modifiers::CTRL, "Ctrl"),
        (Modifiers::ALT, "Alt"),
        (Modifiers::SHIFT, "Shift"),
        (Modifiers::SUPER, "Super"),
    ] {
        if key.modifiers.contains(modifier) {
            if !description.is_empty() {
                description.push('+');
            }
            description.push_str(name);
        }
    }

    if !description.is_empty() {
        description.push('+');
    }
    description.push_str(&keysym_name(keysym));

    description
}

/// Returns the name of a keysym, e.g. `T` for the T key.
fn keysym_name(keysym: Keysym) -> String {
    // xkbcommon returns the canonical name of the keysym, e.g. "t" or "space".
    let name = keysym_get_name(keysym);

    // Single letters are shown in uppercase, the way they are printed on the key.
    if name.len() == 1 {
        name.to_uppercase()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_and_keysym() {
        assert_eq!(
            parse_shortcuts_spec_trigger("CTRL+ALT+T"),
            Some(Key {
                trigger: Trigger::Keysym(Keysym::t),
                modifiers: Modifiers::CTRL | Modifiers::ALT,
            }),
        );
        assert_eq!(
            parse_shortcuts_spec_trigger("CTRL+SHIFT+a"),
            Some(Key {
                trigger: Trigger::Keysym(Keysym::a),
                modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            }),
        );
        assert_eq!(
            parse_shortcuts_spec_trigger("SUPER+space"),
            Some(Key {
                trigger: Trigger::Keysym(Keysym::space),
                modifiers: Modifiers::SUPER,
            }),
        );
        assert_eq!(
            parse_shortcuts_spec_trigger("F8"),
            Some(Key {
                trigger: Trigger::Keysym(Keysym::F8),
                modifiers: Modifiers::empty(),
            }),
        );
    }

    #[test]
    fn ignores_the_num_modifier() {
        assert_eq!(
            parse_shortcuts_spec_trigger("CTRL+NUM+1"),
            Some(Key {
                trigger: Trigger::Keysym(Keysym::_1),
                modifiers: Modifiers::CTRL,
            }),
        );
    }

    #[test]
    fn modifier_names_are_case_insensitive() {
        assert_eq!(
            parse_shortcuts_spec_trigger("ctrl+alt+t"),
            parse_shortcuts_spec_trigger("CTRL+ALT+T"),
        );
        assert_eq!(
            parse_shortcuts_spec_trigger("logo+space"),
            parse_shortcuts_spec_trigger("SUPER+space"),
        );
    }

    #[test]
    fn rejects_invalid_triggers() {
        assert_eq!(parse_shortcuts_spec_trigger("garbage"), None);
        assert_eq!(parse_shortcuts_spec_trigger("CTRL+"), None);
        assert_eq!(parse_shortcuts_spec_trigger(""), None);
        assert_eq!(parse_shortcuts_spec_trigger("CTRL+NotAKey"), None);
        // Two keysyms are not a valid trigger.
        assert_eq!(parse_shortcuts_spec_trigger("CTRL+a+b"), None);
    }

    #[test]
    fn describes_keys() {
        assert_eq!(
            key_description(&Key {
                trigger: Trigger::Keysym(Keysym::t),
                modifiers: Modifiers::CTRL | Modifiers::ALT,
            }),
            "Ctrl+Alt+T",
        );
        assert_eq!(
            key_description(&Key {
                trigger: Trigger::Keysym(Keysym::space),
                modifiers: Modifiers::SUPER,
            }),
            "Super+space",
        );
        assert_eq!(
            key_description(&Key {
                trigger: Trigger::Keysym(Keysym::F8),
                modifiers: Modifiers::empty(),
            }),
            "F8",
        );
    }
}
