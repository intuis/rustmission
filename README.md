# Rustmission

Rustmission is a Terminal User Interface (TUI) client for [Transmission](https://transmissionbt.com) daemon, crafted with async Rust.

## Features

- **Async Rust**: Utilizes Rust's async/await syntax for efficient, non-blocking IO operations.
- **Basic Operations**: Allows users to add, (remove, pause, and resume TODO) torrents.
- **Built-in Search** (Coming Soon): Seamlessly search for torrents directly. This is a killer feature of Rustmission

## Installation TODO

To install Rustmission, ensure you have Rust and Cargo installed on your system, and then run:

```bash
cargo install rustmission # won't work yet
```

## Usage

Launch Rustmission in your terminal to initialize the configuration and make adjustments as needed. Subsequently, run Rustmission again. For list of keybindings, press '?'.

## Configuration

Rustmission stores its configuration in a TOML file located at ~/.config/rustmission/config.toml by default. You can modify this file to
set the daemon's IP address.

```toml
[connection]
url = "http://change_me:9091/transmission/rpc" # REQUIRED!

# If you need username and password to authenticate:
username = "CHANGE_ME"
password = "CHANGE_ME"
```

# Contributing

Contributions are welcome! If you'd like to contribute to Rustmission, please fork the repository, make your changes, and submit a pull request!