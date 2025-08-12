# Termadio

A terminal-based radio player built with Rust using Radio Garden API.

## Prerequisites

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Installation

### Install as Global Command

```bash
# Clone and install globally
git clone <your-repo-url>
cd termadio
cargo install --path .

# Now you can run from anywhere
termadio
```

### Build Only

```bash
# Development build
cargo build
./target/debug/termadio

# Release build (optimized)
cargo build --release
./target/release/termadio
```

## Usage

```bash
# Launch interactive radio terminal
termadio

# Search for stations or countries
termadio search "morocco"

# Show help
termadio --help
```

### Interactive Controls

- **'s'** - Search
- **'f'** - View favorites
- **'c'** - View favorite countries
- **'a'** - Toggle favorite
- **Enter** - Select/play
- **Space** - Pause/resume
- **'x'** - Stop
- **'q'** - Quit

## Data Storage

Data is stored in the current working directory:
- **favorites.json** - Your favorite countries and stations
- **preferences.json** - User preferences

```bash
# View data files
ls -la favorites.json preferences.json
cat favorites.json

# Reset data (delete files)
rm favorites.json preferences.json
```

## Uninstall

```bash
cargo uninstall termadio
```
