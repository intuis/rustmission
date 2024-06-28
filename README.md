<div align="center">
    <h1><strong>Rustmission</strong></h1>
    <p>
	    <strong>TUI for the Transmission daemon</strong>
    </p>
    <img src="https://raw.githubusercontent.com/intuis/rustmission/main/imgs/preview.png" />
    <p>
        <small>⚠️ I DO NOT PIRATE MOVIES. THE TORRENTS YOU SEE IN THIS IMAGE ARE SAMPLE DATA FABRICATED BY ARTIFICIAL INTELLIGENCE. I DO NOT CONDONE PIRACY, AND I WOULD NEVER, EVER DO SUCH THING</small>
    </p>
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

or with Brew:
```bash
brew install intuis/tap/rustmission
```

## Usage

Launch Rustmission in your terminal to initialize the configuration and make adjustments as needed. Subsequently, run Rustmission again. For a list of keybindings, press '?'.

## Configuration

Rustmission stores its configuration in a TOML file located at ~/.config/rustmission/config.toml by default. You can modify this file to
set the daemon's IP address.

```toml
[general]
# Whether to hide empty columns or not
auto_hide = false

# Possible values: Red, Green, Blue, Yellow, Magenta, Cyan.
# Use prefix "Light" for a brighter color.
# It can also be a hex, e.g. "#3cb371"
accent_color = "LightMagenta"

# If enabled, shows various keybindings throughout the program at the cost of a
# little bit cluttered interface.
beginner_mode = true

[connection]
url = "http://CHANGE_ME:9091/transmission/rpc" # REQUIRED!

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
