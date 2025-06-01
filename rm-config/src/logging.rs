use std::fs::{File, OpenOptions};

/// Get an async file writer for the `rustmission.log` file located in XDG state path.
pub fn get_log_file() -> Option<File> {
    match xdg::BaseDirectories::new() {
        Ok(xdg) => {
            let log_path = xdg.get_state_file("rustmission.log");
            match OpenOptions::new().create(true).append(true).open(log_path) {
                Ok(file) => Some(file),
                Err(e) => {
                    eprintln!("Cannot open log file: {e}");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Cannot use XDG path: {e}");
            None
        }
    }
}
