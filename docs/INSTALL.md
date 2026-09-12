# Installing Fonketh

Every [GitHub release](https://github.com/Arvmor/fonketh/releases) ships an
install wizard per platform. Download it, run it, play.

| Platform | Installer |
|---|---|
| macOS (Apple Silicon) | `fonketh-installer-aarch64-apple-darwin` |
| macOS (Intel) | `fonketh-installer-x86_64-apple-darwin` |
| Windows (x64) | `fonketh-installer-x86_64-pc-windows-msvc.exe` |
| Linux (x64) | `fonketh-installer-x86_64-unknown-linux-gnu` |

**Windows:** double-click the `.exe`. SmartScreen will warn that the publisher
is unknown because the binary is not signed: choose *More info* → *Run anyway*.

**macOS / Linux:** the download is not marked executable, so from a terminal:

```sh
chmod +x fonketh-installer-*
./fonketh-installer-*
```

On macOS Gatekeeper may refuse the unsigned binary on first run. Either run it
from the terminal as above, or right-click → *Open* in Finder.

## What the wizard does

1. **System check**: confirms the build matches your OS and CPU.
2. **Release**: looks up the latest tag (or the one you asked for).
3. **Location**: where to install, defaults below.
4. **Download**: fetches the archive, checks it against `SHA256SUMS.txt`,
   unpacks it, installs the `fonketh` command and offers to add it to your PATH.
   On Windows it also creates Start Menu and Desktop shortcuts.
5. **Updates**: whether `fonketh` should update the game automatically before
   each launch (default: yes).
6. **Miner key**: generates a fresh key, imports one you already have, or
   leaves it to the game to create on first launch.

Afterwards open a new terminal and run `fonketh`, or use the shortcut.

## Where things go

| | macOS / Linux | Windows |
|---|---|---|
| Game files | `~/.fonketh/` | `%LOCALAPPDATA%\Programs\Fonketh\` |
| Game binary | `~/.fonketh/bin/fonketh` | `...\Fonketh\bin\fonketh.exe` |
| Textures | `~/.fonketh/assets/` | `...\Fonketh\assets\` |
| Launcher / updater | `~/.fonketh/fonketh-launcher` | `...\Fonketh\fonketh-launcher.exe` |
| Miner key | `~/.fonketh/private.key` | `...\Fonketh\private.key` |
| Update setting | `~/.fonketh/launcher.conf` | `...\Fonketh\launcher.conf` |
| `fonketh` command | `~/.local/bin/fonketh` | `...\Fonketh\fonketh.cmd` |

The `fonketh` command runs `fonketh-launcher launch`, which updates the game
if needed and then starts it from the install directory, so the key and the
per-player sprite composites land there.

**Back up `private.key`.** It is the wallet your `$FONK` rewards go to. Anyone
holding it controls those rewards and nobody can recover it if it is lost.

## Automatic updates

Each time you run `fonketh`, the launcher asks GitHub for the latest release.
If it differs from the installed version it downloads the archive, verifies
the checksum, swaps `bin/` and `assets/` (restoring the old ones if anything
fails) and then starts the game. Your key and settings are never touched. If
GitHub is unreachable the installed version starts as usual.

```sh
fonketh-launcher update            # update now
fonketh-launcher update --check    # only report whether an update exists
fonketh-launcher update --auto off # stop updating on launch (on to re-enable)
fonketh-launcher launch --no-update
```

`FONKETH_AUTO_UPDATE=0` in the environment disables updates for one run.

## Options

The installer is a normal command-line program; run it with `--help`.

```text
fonketh-installer [install] [OPTIONS]
  -y, --yes             Accept all defaults, never prompt
      --version <TAG>   Install a specific release instead of the latest
      --dir <PATH>      Install location
      --bin-dir <PATH>  Where the fonketh command goes (macOS/Linux)
      --archive <FILE>  Install from a downloaded release archive instead of GitHub
      --no-modify-path  Do not touch PATH or shell startup files
      --no-auto-update  Do not update the game automatically on launch

fonketh-installer uninstall [--purge] [-y]
fonketh-installer launch [--no-update] [GAME ARGS...]
fonketh-installer update [--check] [--auto on|off]
```

`FONKETH_YES=1`, `FONKETH_VERSION`, `FONKETH_NO_MODIFY_PATH=1`, `FONKETH_HOME`
and `FONKETH_BIN_DIR` set the same things from the environment.
`FONKETH_RELEASES_URL` points the installer at a GitHub-compatible mirror.

Examples:

```sh
# unattended install of a pinned version
./fonketh-installer-x86_64-unknown-linux-gnu -y --version v0.2.0

# offline install from an archive downloaded on another machine
./fonketh-installer-aarch64-apple-darwin --archive fonketh-v0.2.0-aarch64-apple-darwin.tar.gz
```

## Upgrading

Normally nothing to do: `fonketh` updates before launching. Running the
installer again also works and keeps your key and settings.

## Uninstalling

```sh
~/.fonketh/fonketh-launcher uninstall          # macOS / Linux
%LOCALAPPDATA%\Programs\Fonketh\fonketh-launcher.exe uninstall   # Windows
```

The game files, launcher, shortcuts and PATH entries are removed. The miner
key is kept unless you confirm its deletion (or pass `--purge`).

## Manual install

Download `fonketh-<version>-<target>.tar.gz` (or `.zip` on Windows), unpack it
anywhere and run `bin/fonketh` from the unpacked folder. The game looks for
`assets/` next to the `bin/` directory; set `FONKETH_ASSETS=/path/containing/assets`
to point it elsewhere. Run `./fonketh-launcher launch` from that folder to get
automatic updates.

## Troubleshooting

- **macOS: "cannot be opened because the developer cannot be verified".**
  Nothing is notarized. Run the installer or `fonketh` from a terminal once,
  or right-click → *Open*, or allow it under System Settings → Privacy & Security.
- **Windows: SmartScreen "Windows protected your PC".** *More info* →
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
(with the `interface` feature) and the installer for all four targets, packages
each game as `fonketh-<tag>-<target>.tar.gz` / `.zip` with the launcher
inside, uploads the installers as `fonketh-installer-<target>`, writes
`SHA256SUMS.txt` and publishes the GitHub release. Installed copies pick the
new version up on their next launch.

```sh
git tag v0.2.0
git push origin v0.2.0
```

The workflow can also be re-run for an existing tag from the Actions tab
(*Release* → *Run workflow*). Change the `RELEASE_FEATURES` variable at the
top of the workflow to ship additional features such as `mine`.
