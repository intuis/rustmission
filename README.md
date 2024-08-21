<img align="left" width="100" height="100" src="https://github.com/user-attachments/assets/f3175886-2ccb-4521-a52b-3b9e63687fbb">

**Rustmission**

Performant TUI client for Transmission daemon capable of managing hundreds of torrents.   
It boasts a rich feature set that surpasses many other clients, offering a seamless torrenting experience :3


# 
<div align="center">
    <img src="https://github.com/user-attachments/assets/265c7f7d-85c0-4440-98f8-18c1cbba1a32"/>
    <p>
        <small>⚠️ Torrents you see are just samples fabricated by AI. Piracy is not something we tolerate here at Intuis.</small>
    </p>
</div>

## Features

- **Basic operations**: Allows to add, pause, remove, fuzzy filter your torrents.
- **Built-in magnet search**: Search for magnets without leaving your terminal.
- **Asynchronous**: UI is always responsive.
- **RSS**: Fetch torrents automatically with a cronjob using `--fetch-rss`

## Requirements

- Running [Transmission](https://github.com/transmission/transmission) daemon and its IP address
- [Nerd Fonts](https://www.nerdfonts.com/)

## Installation

<a href="https://repology.org/project/rustmission/versions">
    <img src="https://repology.org/badge/vertical-allrepos/rustmission.svg" alt="Packaging status" align="right">
</a>

To install Rustmission, ensure you have Rust and Cargo installed on your system, and then run:

```bash
cargo install rustmission
```

or with Nix ( :heart: [@0x61nas](https://github.com/0x61nas) ):

```bash
nix-shell -p rustmission
```

or with Brew ( :heart: [@aidanaden](https://github.com/aidanaden) ):
```bash
brew install intuis/tap/rustmission
```

## Usage

Run `rustmission` in your terminal to initialize the config and make adjustments as needed. Subsequently, run `rustmission` again. For a list of keybindings, press `?` or `F1`.

## Configuration

Rustmission stores its configuration in a TOML file located at `~/.config/rustmission/config.toml` by default. You can modify this file to
set the daemon's address.

```toml
[general]
# Whether to hide empty columns or not
auto_hide = false

# Possible values: Red, Green, Blue, Yellow, Magenta, Cyan.
# Use prefix "Light" for a brighter color.
# It can also be a hex, e.g. "#3cb371"
accent_color = "LightMagenta"

# If enabled, shows various keybindings throughout the program at the cost of
# a little bit cluttered interface.
beginner_mode = true

# If enabled, hides header row of torrents tab
headers_hide = false

[connection]
url = "http://CHANGE_ME:9091/transmission/rpc" # REQUIRED!

# Refresh timings (in seconds)
torrents_refresh = 5
stats_refresh = 5
free_space_refresh = 10

# If you need username and password to authenticate:
# username = "CHANGE_ME"
# password = "CHANGE_ME"

[torrents_tab]
# Available fields:
# Id, Name, SizeWhenDone, Progress, DownloadRate, UploadRate, DownloadDir,
# Padding, UploadRatio, UploadedEver, AddedDate, ActivityDate, PeersConnected
# SmallStatus
headers = ["Name", "SizeWhenDone", "Progress", "DownloadRate", "UploadRate"]

```

There's also a self-documenting keymap config located at `~/.config/rustmission/keymap.toml` with sane defaults.

## Alternatives
- [Transgression](https://github.com/PanAeon/transg-tui)
- [tremc](https://github.com/tremc/tremc)
- [trt](https://github.com/murtaza-u/transmission-remote-tui)
- [stig](https://github.com/rndusr/stig)

## Contributing
If you'd like to contribute make sure you fork [this repo](https://github.com/intuis/rustmission) and submit a PR!  
If you want to implement something major, create an issue first so it can be discussed.
