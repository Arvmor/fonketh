# Installing Fonketh

Fonketh ships prebuilt binaries for macOS (Apple Silicon and Intel), Windows
(x64) and Linux (x64) on the [GitHub Releases](https://github.com/Arvmor/fonketh/releases)
page. The installer scripts in the repository root download the right one,
verify it and set everything up.

## Quick start

**macOS / Linux**

```sh
curl -fsSL https://raw.githubusercontent.com/Arvmor/fonketh/master/install.sh | sh
```

**Windows** (PowerShell 5.1 or newer)

```powershell
irm https://raw.githubusercontent.com/Arvmor/fonketh/master/install.ps1 | iex
```

The wizard walks through five steps:

1. **System check**: detects your OS and CPU and picks the matching build.
2. **Release**: looks up the latest tag (or the one you asked for).
3. **Location**: where to install, defaults below.
4. **Download**: fetches the archive, checks it against `SHA256SUMS.txt`,
   unpacks it, installs a `fonketh` launcher and offers to add it to your PATH.
5. **Miner key**: generates a fresh key, imports one you already have, or
   leaves it to the game to create on first launch.

Afterwards open a new terminal and run `fonketh`. On Windows there is also a
Start Menu shortcut.

## Where things go

| | macOS / Linux | Windows |
|---|---|---|
| Game files | `~/.fonketh/` | `%LOCALAPPDATA%\Programs\Fonketh\` |
| Binary | `~/.fonketh/bin/fonketh` | `...\Fonketh\bin\fonketh.exe` |
| Textures | `~/.fonketh/assets/` | `...\Fonketh\assets\` |
| Miner key | `~/.fonketh/private.key` | `...\Fonketh\private.key` |
| Launcher | `~/.local/bin/fonketh` | `...\Fonketh\fonketh.cmd` |

The launcher starts the game from the install directory so it finds the key
and can write the per-player sprite composites next to the textures.

**Back up `private.key`.** It is the wallet your `$FONK` rewards go to. Anyone
holding it controls those rewards and nobody can recover it if it is lost.

## Options

Both scripts accept the same settings, as flags when you run a downloaded
copy, or as environment variables when piping from `curl` / `irm`.

| Setting | `install.sh` | `install.ps1` | Environment |
|---|---|---|---|
| Unattended, accept defaults | `-y` | `-Yes` | `FONKETH_YES=1` |
| Specific release | `--version v0.2.0` | `-Version v0.2.0` | `FONKETH_VERSION` |
| Install directory | `--dir PATH` | `-InstallDir PATH` | `FONKETH_HOME` |
| Launcher directory | `--bin-dir PATH` | | `FONKETH_BIN_DIR` |
| Leave PATH alone | `--no-modify-path` | `-NoModifyPath` | `FONKETH_NO_MODIFY_PATH=1` |
| Install from a local archive | `--archive FILE` | `-Archive FILE` | |
| Uninstall | `--uninstall [--purge]` | `-Uninstall [-Purge]` | `FONKETH_UNINSTALL=1` |

Examples:

```sh
# pin a version, unattended
curl -fsSL https://raw.githubusercontent.com/Arvmor/fonketh/master/install.sh | sh -s -- --version v0.2.0 -y

# offline install from an archive downloaded on another machine
sh install.sh --archive fonketh-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
```

```powershell
# pin a version, unattended
$env:FONKETH_VERSION = 'v0.2.0'; $env:FONKETH_YES = '1'
irm https://raw.githubusercontent.com/Arvmor/fonketh/master/install.ps1 | iex
```

## Upgrading

Run the installer again. It replaces `bin/` and `assets/`, keeps your
`private.key`, and skips the download when the installed version already
matches the latest release.

## Uninstalling

```sh
curl -fsSL https://raw.githubusercontent.com/Arvmor/fonketh/master/install.sh | sh -s -- --uninstall
```

```powershell
$env:FONKETH_UNINSTALL = '1'
irm https://raw.githubusercontent.com/Arvmor/fonketh/master/install.ps1 | iex
```

The game files, launcher, shortcuts and PATH entries are removed. The miner
key is kept unless you confirm its deletion (or pass `--purge` / `-Purge`).

## Manual install

Download `fonketh-<version>-<target>.tar.gz` (or `.zip` on Windows) from the
release page, unpack it anywhere and run `bin/fonketh` from the unpacked
folder. The game looks for `assets/` next to the `bin/` directory; set
`FONKETH_ASSETS=/path/containing/assets` to point it elsewhere.

| Platform | Target |
|---|---|
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Windows x64 | `x86_64-pc-windows-msvc` |
| Linux x64 | `x86_64-unknown-linux-gnu` |

## Troubleshooting

- **macOS: "cannot be opened because the developer cannot be verified".**
  The binaries are not notarized. The installer strips the quarantine flag;
  if Gatekeeper still objects, run `fonketh` from a terminal once, or allow it
  under System Settings → Privacy & Security.
- **Windows: SmartScreen "Windows protected your PC".** Click *More info* →
  *Run anyway*. The binaries are not code-signed.
- **Firewall.** Peers reach you over UDP port 7331 (QUIC). Allow the prompt on
  first launch, or forward the port on your router for faster discovery. The
  local API listens on TCP 8080.
- **Linux: missing libraries.** The game needs ALSA, udev, Wayland/X11 and
  xkbcommon runtime libraries, e.g. on Debian/Ubuntu
  `sudo apt install libasound2 libudev1 libwayland-client0 libxkbcommon0`.
- **"release vX has no build for <target>".** Releases before v0.2.0 shipped
  bare binaries without the archive layout the installer expects. Pick a newer
  release with `--version`, or build from source:

  ```sh
  git clone https://github.com/Arvmor/fonketh && cd fonketh
  cargo run -p game_app -F interface --release
  ```

## Cutting a release (maintainers)

Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds the game
with the `interface` feature for all four targets, packages each as
`fonketh-<tag>-<target>.tar.gz` / `.zip`, writes `SHA256SUMS.txt` and
publishes a GitHub release with install instructions.

```sh
git tag v0.2.0
git push origin v0.2.0
```

The workflow can also be re-run for an existing tag from the Actions tab
(*Release* → *Run workflow*). Change the `RELEASE_FEATURES` variable at the
top of the workflow to ship additional features such as `mine`.
