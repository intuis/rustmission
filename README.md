<div align="center">
    <h1><strong>Rustmission</strong></h1>
    <p>
	    <strong>TUI for the Transmission daemon</strong>
    </p>
    <a href="https://asciinema.org/a/5szjGR0ccoYolnV3zP3b6vqxF" target="_blank"><img src="https://asciinema.org/a/5szjGR0ccoYolnV3zP3b6vqxF.svg" /></a>
</div>

## Features

- **Built-in Search**: Seamlessly search for magnets directly. This is a killer feature of Rustmission.
- **Async Rust**: Utilizes Rust's async/await syntax for efficient, non-blocking IO operations.
- **Basic Operations**: Allows to add, pause, remove, fuzzy filter your torrents.

## Installation

To install Rustmission, ensure you have Rust and Cargo installed on your system, and then run:

```bash
cargo install rustmission
```

or with Nix:

```bash
nix run .
```

## Usage

Launch Rustmission in your terminal to initialize the configuration and make adjustments as needed. Subsequently, run Rustmission again. For list of keybindings, press '?'.

## Configuration

Rustmission stores its configuration in a TOML file located at ~/.config/rustmission/config.toml by default. You can modify this file to
set the daemon's IP address.

```toml
[general]
# Whether to hide empty columns or not
# auto_hide = false

# Possible values: Red, Green, Blue, Yellow, Magenta, Cyan.
# Use prefix "Light" for a brighter color.
# accent_color = "LightMagenta"

[connection]
url = "http://change_me:9091/transmission/rpc" # REQUIRED!

# If you need username and password to authenticate:
# username = "CHANGE_ME"
# password = "CHANGE_ME"
```

## Alternatives
- [Transgression](https://github.com/PanAeon/transg-tui)
- [tremc](https://github.com/tremc/tremc)
- [trt](https://github.com/murtaza-u/transmission-remote-tui)
- [stig](https://github.com/rndusr/stig)

## Contributing

Contributions are welcome! If you'd like to contribute to Rustmission, please fork the repository, make your changes, and submit a pull request!
