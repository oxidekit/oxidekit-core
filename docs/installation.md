# Installation

## Quick Install (Recommended)

### macOS / Linux

```bash
curl -fsSL https://oxidekit.com/install.sh | bash
```

Then add to your PATH:
```bash
export PATH="$HOME/.oxide/bin:$PATH"
```

### Homebrew (macOS / Linux)

```bash
brew install oxidekit/tap/oxide
```

### Windows

Download the latest `.zip` from [GitHub Releases](https://github.com/oxidekit/oxidekit-core/releases) and add to your PATH.

Or use scoop:
```powershell
scoop bucket add oxidekit https://github.com/oxidekit/scoop-bucket
scoop install oxide
```

## From Source

Requires Rust 1.75+:

```bash
cargo install oxide-cli
```

Or build from repository:

```bash
git clone https://github.com/oxidekit/oxidekit-core
cd oxidekit-core
cargo install --path crates/oxide-cli
```

## Verify Installation

```bash
oxide --version
# oxide 0.3.0
```

## System Requirements

### macOS
- macOS 11.0 (Big Sur) or later
- Apple Silicon (M1/M2/M3) or Intel x86_64

### Linux
- glibc 2.17 or later (most distros from 2014+)
- x86_64 architecture
- GPU with Vulkan support (recommended)

### Windows
- Windows 10 or later
- x86_64 architecture
- DirectX 12 or Vulkan support

## Updating

### Quick Install

Re-run the install script:
```bash
curl -fsSL https://oxidekit.com/install.sh | bash
```

### Homebrew

```bash
brew upgrade oxide
```

### Cargo

```bash
cargo install oxide-cli --force
```

## Uninstalling

### Quick Install

```bash
rm -rf ~/.oxide
```

Remove the PATH line from your shell config.

### Homebrew

```bash
brew uninstall oxide
```

### Cargo

```bash
cargo uninstall oxide-cli
```

## Troubleshooting

### "command not found: oxide"

Ensure `~/.oxide/bin` is in your PATH:
```bash
echo $PATH | grep -q ".oxide/bin" && echo "OK" || echo "Add to PATH"
```

### GPU not detected

OxideKit requires GPU support. Check:
```bash
oxide doctor
```

### Permission denied

On macOS, you may need to allow the binary:
```bash
xattr -d com.apple.quarantine ~/.oxide/bin/oxide
```
