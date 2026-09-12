#!/bin/sh
# Fonketh installer for macOS and Linux
#
# Downloads the latest release from GitHub, unpacks it into $FONKETH_HOME
# (default ~/.fonketh), puts a `fonketh` launcher on your PATH and helps you
# set up a miner key.
#
#   curl -fsSL https://raw.githubusercontent.com/Arvmor/fonketh/master/install.sh | sh
#
# Run with -h for options. Every prompt has a sensible default, so `-y`
# performs a fully unattended install.

set -eu

REPO="${FONKETH_REPO:-Arvmor/fonketh}"
FONKETH_HOME="${FONKETH_HOME:-$HOME/.fonketh}"
BIN_DIR="${FONKETH_BIN_DIR:-$HOME/.local/bin}"
VERSION="${FONKETH_VERSION:-}"
ARCHIVE=""
# Env toggles let `curl | sh` users pick options without `sh -s --`
ASSUME_YES=0;   [ "${FONKETH_YES:-0}" = "1" ] && ASSUME_YES=1
MODIFY_PATH=1;  [ "${FONKETH_NO_MODIFY_PATH:-0}" = "1" ] && MODIFY_PATH=0
UNINSTALL=0;    [ "${FONKETH_UNINSTALL:-0}" = "1" ] && UNINSTALL=1
PURGE=0

# ---------------------------------------------------------------- output ----

if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD="$(printf '\033[1m')"; DIM="$(printf '\033[2m')"
    GREEN="$(printf '\033[32m')"; YELLOW="$(printf '\033[33m')"
    RED="$(printf '\033[31m')"; CYAN="$(printf '\033[36m')"
    RESET="$(printf '\033[0m')"
else
    BOLD=""; DIM=""; GREEN=""; YELLOW=""; RED=""; CYAN=""; RESET=""
fi

say()  { printf '%s\n' "$*"; }
step() { printf '\n%s==>%s %s%s%s\n' "$CYAN" "$RESET" "$BOLD" "$*" "$RESET"; }
ok()   { printf '  %s✓%s %s\n' "$GREEN" "$RESET" "$*"; }
note() { printf '  %s%s%s\n' "$DIM" "$*" "$RESET"; }
warn() { printf '  %s!%s %s\n' "$YELLOW" "$RESET" "$*" >&2; }
die()  { printf '\n%serror:%s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }

banner() {
    printf '%s' "$CYAN"
    cat <<'BANNER'
   ___          _        _   _
  | __|__ _ __ | |__ ___| |_| |__
  | _/ _ \ ' \ | / // -_)  _| ' \
  |_|\___/_||_||_\_\\___|\__|_||_|
BANNER
    printf '%s' "$RESET"
    say "  P2P mining pool, gamified. https://github.com/$REPO"
}

usage() {
    cat <<USAGE
Fonketh installer

Usage: install.sh [options]

Options:
  -y, --yes             Accept all defaults, never prompt
      --version <tag>   Install a specific release (e.g. v0.2.0) instead of the latest
      --dir <path>      Install location (default: $FONKETH_HOME)
      --bin-dir <path>  Where the \`fonketh\` launcher goes (default: $BIN_DIR)
      --archive <file>  Install from a downloaded release archive instead of GitHub
      --no-modify-path  Do not touch shell startup files
      --uninstall       Remove the game (keeps your private key unless --purge)
      --purge           With --uninstall, also delete the private key
  -h, --help            Show this help

Environment:
  FONKETH_HOME, FONKETH_BIN_DIR, FONKETH_VERSION, FONKETH_REPO   same as the flags above
  FONKETH_YES=1, FONKETH_NO_MODIFY_PATH=1, FONKETH_UNINSTALL=1  same as -y, --no-modify-path, --uninstall
  NO_COLOR
USAGE
}

# --------------------------------------------------------------- prompts ----

# Reads a line from the terminal even when stdin is the pipe from curl.
read_tty() {
    if [ -r /dev/tty ]; then
        IFS= read -r REPLY < /dev/tty || REPLY=""
    else
        IFS= read -r REPLY || REPLY=""
    fi
}

interactive() {
    [ "$ASSUME_YES" -eq 0 ] && [ -r /dev/tty ] && [ -t 2 ]
}

# ask "Question" "default" -> $REPLY
ask() {
    REPLY="$2"
    if interactive; then
        printf '  %s%s%s [%s]: ' "$BOLD" "$1" "$RESET" "$2" >&2
        read_tty
        [ -n "$REPLY" ] || REPLY="$2"
    fi
}

# confirm "Question" [Y|N] -> exit status
confirm() {
    default="${2:-Y}"
    if ! interactive; then
        [ "$default" = "Y" ]
        return
    fi
    if [ "$default" = "Y" ]; then hint="Y/n"; else hint="y/N"; fi
    printf '  %s%s%s [%s]: ' "$BOLD" "$1" "$RESET" "$hint" >&2
    read_tty
    case "${REPLY:-$default}" in
        [Yy]*) return 0 ;;
        *) return 1 ;;
    esac
}

# choose "Question" "default" "1) ..." "2) ..." ... -> $REPLY (number)
choose() {
    question="$1"; default="$2"; shift 2
    for option in "$@"; do
        printf '    %s\n' "$option" >&2
    done
    ask "$question" "$default"
}

# --------------------------------------------------------------- helpers ----

need() {
    command -v "$1" > /dev/null 2>&1 || die "'$1' is required but not installed"
}

download() {
    # download <url> <dest>
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL --retry 3 --proto '=https' --tlsv1.2 -o "$2" "$1"
    elif command -v wget > /dev/null 2>&1; then
        wget -q -O "$2" "$1"
    else
        die "curl or wget is required to download the game"
    fi
}

sha256() {
    if command -v sha256sum > /dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum > /dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        return 1
    fi
}

random_hex32() {
    if command -v openssl > /dev/null 2>&1; then
        openssl rand -hex 32
    else
        od -An -N32 -tx1 /dev/urandom | tr -d ' \n'
    fi
}

detect_target() {
    os="$(uname -s)"; arch="$(uname -m)"
    case "$os" in
        Darwin)
            case "$arch" in
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
                x86_64) TARGET="x86_64-apple-darwin" ;;
                *) die "unsupported macOS architecture: $arch" ;;
            esac ;;
        Linux)
            case "$arch" in
                x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
                *) die "unsupported Linux architecture: $arch (build from source: cargo run -p game_app -F interface --release)" ;;
            esac ;;
        MINGW*|MSYS*|CYGWIN*)
            die "on Windows use PowerShell: irm https://raw.githubusercontent.com/$REPO/master/install.ps1 | iex" ;;
        *) die "unsupported operating system: $os" ;;
    esac
    OS="$os"
}

# Resolves the release tag without the GitHub API (no rate limits):
# github.com/<repo>/releases/latest redirects to /releases/tag/<tag>.
resolve_version() {
    [ -n "$VERSION" ] && return
    need curl
    location="$(curl -fsSI -o /dev/null -w '%{redirect_url}' "https://github.com/$REPO/releases/latest")" \
        || die "could not reach GitHub to look up the latest release"
    VERSION="${location##*/tag/}"
    case "$VERSION" in
        v*) ;;
        *) die "could not determine the latest release (got '$location')" ;;
    esac
}

# ---------------------------------------------------------------- install ---

fetch_archive() {
    # Sets $ARCHIVE to a local .tar.gz, downloading and verifying it when needed
    name="fonketh-$VERSION-$TARGET"
    if [ -n "$ARCHIVE" ]; then
        [ -f "$ARCHIVE" ] || die "archive not found: $ARCHIVE"
        ok "Using local archive $ARCHIVE"
        return
    fi

    base="https://github.com/$REPO/releases/download/$VERSION"
    ARCHIVE="$TMP/$name.tar.gz"
    say "  Downloading $name.tar.gz"
    if ! download "$base/$name.tar.gz" "$ARCHIVE"; then
        die "release $VERSION has no build for $TARGET.
  Releases older than the installer shipped bare binaries; pick a newer one with
  --version, or build from source:
    git clone https://github.com/$REPO && cd fonketh && cargo run -p game_app -F interface --release"
    fi
    ok "Downloaded $(du -h "$ARCHIVE" | cut -f1 | tr -d ' ')"

    if download "$base/SHA256SUMS.txt" "$TMP/SHA256SUMS.txt"; then
        expected="$(grep " $name.tar.gz\$" "$TMP/SHA256SUMS.txt" | cut -d' ' -f1)"
        actual="$(sha256 "$ARCHIVE" || true)"
        if [ -z "$actual" ]; then
            warn "No sha256 tool found, skipping checksum verification"
        elif [ -z "$expected" ]; then
            warn "Archive missing from SHA256SUMS.txt, skipping checksum verification"
        elif [ "$expected" != "$actual" ]; then
            die "checksum mismatch for $name.tar.gz
  expected $expected
  got      $actual"
        else
            ok "Checksum verified"
        fi
    else
        warn "Release has no SHA256SUMS.txt, skipping checksum verification"
    fi
}

install_files() {
    mkdir -p "$TMP/unpack"
    tar -xzf "$ARCHIVE" -C "$TMP/unpack"
    src="$(find "$TMP/unpack" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
    [ -n "$src" ] && [ -x "$src/bin/fonketh" ] || die "unexpected archive layout (no bin/fonketh)"
    [ -d "$src/assets" ] || die "unexpected archive layout (no assets/)"

    mkdir -p "$FONKETH_HOME"
    # Replace the game files, keep everything else (private key, logs, ...)
    rm -rf "$FONKETH_HOME/bin" "$FONKETH_HOME/assets"
    cp -R "$src/bin" "$src/assets" "$FONKETH_HOME/"
    [ -f "$src/README.md" ] && cp "$src/README.md" "$FONKETH_HOME/"
    [ -f "$src/VERSION" ] && cp "$src/VERSION" "$FONKETH_HOME/" || printf '%s\n' "$VERSION" > "$FONKETH_HOME/VERSION"
    chmod +x "$FONKETH_HOME/bin/fonketh"
    ok "Game files installed to $FONKETH_HOME"

    if [ "$OS" = "Darwin" ] && command -v xattr > /dev/null 2>&1; then
        xattr -dr com.apple.quarantine "$FONKETH_HOME/bin/fonketh" 2> /dev/null || true
    fi
}

install_launcher() {
    mkdir -p "$BIN_DIR"
    cat > "$BIN_DIR/fonketh" <<LAUNCHER
#!/bin/sh
# Fonketh launcher, generated by install.sh
# The game keeps its private key and sprites next to itself, so run it from there.
FONKETH_HOME="\${FONKETH_HOME:-$FONKETH_HOME}"
cd "\$FONKETH_HOME" || exit 1
exec "\$FONKETH_HOME/bin/fonketh" "\$@"
LAUNCHER
    chmod +x "$BIN_DIR/fonketh"
    ok "Launcher installed to $BIN_DIR/fonketh"
}

on_path() {
    case ":$PATH:" in
        *":$1:"*) return 0 ;;
        *) return 1 ;;
    esac
}

setup_path() {
    on_path "$BIN_DIR" && { ok "$BIN_DIR is already on your PATH"; return; }

    # rustup-style env file that startup scripts source
    cat > "$FONKETH_HOME/env" <<ENV
#!/bin/sh
# Fonketh: add the launcher directory to PATH
case ":\${PATH}:" in
    *:"$BIN_DIR":*) ;;
    *) export PATH="$BIN_DIR:\$PATH" ;;
esac
ENV
    line=". \"$FONKETH_HOME/env\""

    if [ "$MODIFY_PATH" -eq 0 ]; then
        warn "$BIN_DIR is not on your PATH. Add this to your shell startup file:"
        say "      $line"
        return
    fi

    if ! confirm "Add $BIN_DIR to your PATH (edits shell startup files)?" Y; then
        warn "Skipped. To run the game later add this to your shell startup file:"
        say "      $line"
        return
    fi

    updated=""
    for rc in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.zshenv"; do
        [ -f "$rc" ] || continue
        grep -Fq "$FONKETH_HOME/env" "$rc" 2> /dev/null && continue
        printf '\n%s\n' "$line" >> "$rc"
        updated="$updated $rc"
    done
    # zsh users without a .zshrc still get one
    if [ -z "$updated" ] && [ "${SHELL##*/}" = "zsh" ]; then
        printf '\n%s\n' "$line" >> "$HOME/.zshrc"; updated=" $HOME/.zshrc"
    fi
    if [ -z "$updated" ]; then
        printf '\n%s\n' "$line" >> "$HOME/.profile"; updated=" $HOME/.profile"
    fi
    fish_conf="${XDG_CONFIG_HOME:-$HOME/.config}/fish/conf.d"
    if command -v fish > /dev/null 2>&1; then
        mkdir -p "$fish_conf"
        printf 'fish_add_path --prepend --global "%s"\n' "$BIN_DIR" > "$fish_conf/fonketh.fish"
    fi
    ok "PATH updated in:$updated"
    note "Open a new terminal, or run: $line"
}

setup_key() {
    key_file="$FONKETH_HOME/private.key"
    if [ -f "$key_file" ]; then
        ok "Keeping the existing miner key at $key_file"
        return
    fi

    say "  Your rewards go to the wallet derived from this key."
    choose "Miner key" 1 \
        "1) Generate a new key now (recommended)" \
        "2) Import an existing private key" \
        "3) Skip, the game creates one on first launch"
    case "$REPLY" in
        2)
            if ! interactive; then die "--yes cannot import a key"; fi
            printf '  Paste the 32-byte hex private key (input hidden): ' >&2
            stty -echo < /dev/tty 2> /dev/null || true
            read_tty; key="$REPLY"
            stty echo < /dev/tty 2> /dev/null || true
            printf '\n' >&2
            key="$(printf '%s' "$key" | tr -d ' \r\n')"
            case "$key" in 0x*|0X*) key="${key#0?}" ;; esac
            [ "${#key}" -eq 64 ] && [ -z "$(printf '%s' "$key" | tr -d '0-9a-fA-F')" ] \
                || die "that is not a 64-character hex private key"
            ;;
        3)
            note "The game will write $key_file on first launch"
            return ;;
        *)
            key="$(random_hex32)"
            ;;
    esac
    umask 077
    printf '0x%s' "$key" > "$key_file"
    chmod 600 "$key_file"
    ok "Miner key saved to $key_file"
    warn "Back this file up. Anyone with it controls your rewards; nobody can recover it for you."
}

summary() {
    step "Fonketh $VERSION is installed"
    say "  Game files : $FONKETH_HOME"
    say "  Launcher   : $BIN_DIR/fonketh"
    say "  Miner key  : $FONKETH_HOME/private.key"
    say ""
    say "  Run ${BOLD}fonketh${RESET} to play. Peers find you faster if UDP port 7331 is open."
    if [ "$OS" = "Darwin" ]; then
        note "macOS: the binary is not notarized. If Gatekeeper blocks it, run it from the terminal once."
    fi
    case "$0" in
        */install.sh|install.sh) say "  Uninstall with: $0 --uninstall" ;;
        *) say "  Uninstall with: curl -fsSL https://raw.githubusercontent.com/$REPO/master/install.sh | sh -s -- --uninstall" ;;
    esac

    if interactive && confirm "Launch Fonketh now?" N; then
        exec "$BIN_DIR/fonketh"
    fi
}

uninstall() {
    banner
    step "Uninstalling Fonketh"
    [ -d "$FONKETH_HOME" ] || [ -f "$BIN_DIR/fonketh" ] || die "nothing to uninstall at $FONKETH_HOME"
    confirm "Remove $FONKETH_HOME/{bin,assets} and $BIN_DIR/fonketh?" Y || exit 0
    rm -rf "$FONKETH_HOME/bin" "$FONKETH_HOME/assets" "$FONKETH_HOME/env" \
        "$FONKETH_HOME/README.md" "$FONKETH_HOME/VERSION" "$BIN_DIR/fonketh"
    rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/fish/conf.d/fonketh.fish"
    for rc in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.zshenv"; do
        [ -f "$rc" ] || continue
        grep -Fq "$FONKETH_HOME/env" "$rc" || continue
        grep -Fv "$FONKETH_HOME/env" "$rc" > "$rc.fonketh-tmp" && mv "$rc.fonketh-tmp" "$rc"
    done
    ok "Game files removed"
    if [ "$PURGE" -eq 1 ] || { [ -f "$FONKETH_HOME/private.key" ] && confirm "Also delete the miner key $FONKETH_HOME/private.key? This cannot be undone" N; }; then
        rm -f "$FONKETH_HOME/private.key"
        ok "Miner key deleted"
    else
        [ -f "$FONKETH_HOME/private.key" ] && note "Kept $FONKETH_HOME/private.key"
    fi
    rmdir "$FONKETH_HOME" 2> /dev/null || true
    say ""
}

# ------------------------------------------------------------------ main ----

while [ $# -gt 0 ]; do
    case "$1" in
        -y|--yes) ASSUME_YES=1 ;;
        --version) shift; VERSION="${1:-}"; [ -n "$VERSION" ] || die "--version needs a tag" ;;
        --version=*) VERSION="${1#*=}" ;;
        --dir) shift; FONKETH_HOME="${1:-}"; [ -n "$FONKETH_HOME" ] || die "--dir needs a path" ;;
        --dir=*) FONKETH_HOME="${1#*=}" ;;
        --bin-dir) shift; BIN_DIR="${1:-}"; [ -n "$BIN_DIR" ] || die "--bin-dir needs a path" ;;
        --bin-dir=*) BIN_DIR="${1#*=}" ;;
        --archive) shift; ARCHIVE="${1:-}"; [ -n "$ARCHIVE" ] || die "--archive needs a file" ;;
        --archive=*) ARCHIVE="${1#*=}" ;;
        --no-modify-path) MODIFY_PATH=0 ;;
        --uninstall) UNINSTALL=1 ;;
        --purge) PURGE=1 ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; die "unknown option: $1" ;;
    esac
    shift
done

case "$VERSION" in ""|v*) ;; *) VERSION="v$VERSION" ;; esac

if [ "$UNINSTALL" -eq 1 ]; then
    uninstall
    exit 0
fi

TMP="$(mktemp -d 2> /dev/null || mktemp -d -t fonketh)"
# Also restore echo in case the hidden key prompt was interrupted
cleanup() {
    [ -r /dev/tty ] && stty echo < /dev/tty 2> /dev/null
    rm -rf "$TMP"
}
trap 'cleanup' EXIT
trap 'cleanup; exit 130' INT
trap 'cleanup; exit 143' TERM

banner
need tar
detect_target

step "Step 1/5  Checking your system"
ok "Detected $OS ($TARGET)"

step "Step 2/5  Picking a release"
if [ -z "$ARCHIVE" ]; then
    resolve_version
    ok "Release $VERSION"
    if [ -f "$FONKETH_HOME/VERSION" ] && [ "$(cat "$FONKETH_HOME/VERSION")" = "$VERSION" ]; then
        ok "Already installed at $FONKETH_HOME"
        if ! confirm "Reinstall anyway?" N; then exit 0; fi
    fi
else
    if [ -z "$VERSION" ]; then
        # fonketh-<tag>-<target>.tar.gz, tags may contain dashes so strip the known target
        VERSION="$(basename "$ARCHIVE" .tar.gz)"; VERSION="${VERSION#fonketh-}"; VERSION="${VERSION%-"$TARGET"}"
        case "$VERSION" in v*) ;; *) VERSION="local" ;; esac
    fi
    ok "Release $VERSION (local archive)"
fi

step "Step 3/5  Choosing where to install"
ask "Install directory" "$FONKETH_HOME"; FONKETH_HOME="$REPLY"
ask "Launcher directory" "$BIN_DIR"; BIN_DIR="$REPLY"
case "$FONKETH_HOME" in /*) ;; *) FONKETH_HOME="$PWD/$FONKETH_HOME" ;; esac
case "$BIN_DIR" in /*) ;; *) BIN_DIR="$PWD/$BIN_DIR" ;; esac
if interactive; then
    confirm "Install Fonketh $VERSION to $FONKETH_HOME?" Y || { say "  Aborted."; exit 0; }
else
    ok "Installing to $FONKETH_HOME"
fi

step "Step 4/5  Downloading and installing"
fetch_archive
install_files
install_launcher
setup_path

step "Step 5/5  Miner key"
setup_key

summary
