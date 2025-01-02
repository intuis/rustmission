<img align="left" width="100" height="100" src="https://github.com/user-attachments/assets/29ce0441-889e-42d7-b9ce-3f1c9888ef31">

**Rustmission**

Performant TUI for Transmission capable of managing hundreds of torrents.   
It boasts a rich feature set that surpasses other clients, offering a seamless experience :3


# 
<div align="center">
    <img src="https://github.com/user-attachments/assets/d6a7b35d-f6b4-408f-bf9e-1cf4ddc9914c"/>
    <p>
        <small>Torrents you see are just samples fabricated by AI.</small>
    </p>
</div>

## Features

- **Basic operations**: Add, pause, remove or fuzzy filter your torrents.
- **Categories**: Categorize your torrents, each with its own default directory.
- **Sorting**: Sort your torrents, for example, to get rid of the bulkiest ones.
- **Built-in magnet search**: Search for new magnets without leaving your terminal.
- **Asynchronous**: UI is always responsive.
- **RSS**: Fetch torrents automatically with a cronjob using `--fetch-rss`

## Requirements

- [Transmission](https://github.com/transmission/transmission) daemon running in background and its IP address
- [Nerd Fonts](https://www.nerdfonts.com/) (though you can set alternative set of icons)

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
# SmallStatus, Category, CategoryIcon
headers = ["Name", "SizeWhenDone", "Progress", "Eta", "DownloadRate", "UploadRate"]

[search_tab]
# If you uncomment this, providers won't be automatically added in future
# versions of Rustmission.
# providers = ["Knaben", "Nyaa"]

[icons]
# ...

```

There's also a self-documenting keymap config located at `~/.config/rustmission/keymap.toml` with sane defaults.  
You can also define torrent categories at `~/.config/rustmission/categories.toml`.

## Alternatives
- [Transgression](https://github.com/PanAeon/transg-tui)
- [tremc](https://github.com/tremc/tremc)
- [trt](https://github.com/murtaza-u/transmission-remote-tui)
- [stig](https://github.com/rndusr/stig)

## Contributing
If you'd like to contribute make sure you fork [this repo](https://github.com/intuis/rustmission) and submit a PR!  
If you want to implement something major, create an issue first so it can be discussed.
