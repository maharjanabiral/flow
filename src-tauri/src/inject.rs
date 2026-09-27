/// Types text into the currently focused window. `prepend_space` controls
/// whether a leading space is added first — pass `false` only for the very
/// first dictation of a session, `true` for every subsequent one, so
/// consecutive dictations are naturally separated by a space.
pub fn type_text(text: &str, prepend_space: bool) {
    if text.is_empty() {
        return;
    }

    let output = if prepend_space {
        format!(" {}", text)
    } else {
        text.to_string()
    };

    #[cfg(target_os = "linux")]
    {
        type_text_linux(&output);
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        type_text_other(&output);
    }
}

#[cfg(target_os = "linux")]
fn type_text_linux(text: &str) {
    use std::process::Command;

    let result = Command::new("ydotool").arg("type").arg(text).status();

    match result {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("ydotool exited with status: {status}"),
        Err(e) => {
            eprintln!("failed to run ydotool: {e} — is it installed and is ydotoold running?")
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn type_text_other(text: &str) {
    use enigo::{Enigo, Keyboard, Settings};

    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("failed to initialize enigo: {e}");
            return;
        }
    };

    if let Err(e) = enigo.text(text) {
        eprintln!("failed to type text: {e}");
    }
}
