//! Fonketh install wizard and launcher
//!
//! Run without arguments it walks through installing the latest release:
//! download, verify, unpack, launcher on PATH, auto-update preference and
//! miner key. Every prompt has a default, so `--yes` performs an unattended
//! install.
//!
//! The installer also copies itself into the game's home as
//! `fonketh-launcher`. The `fonketh` command runs `fonketh-launcher launch`,
//! which brings the game up to date and then starts it.

mod key;
mod layout;
mod release;
mod ui;
mod update;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use console::style;
use layout::{Layout, platform};
use release::{GitHub, Source, TARGET};
use std::path::PathBuf;
use ui::Ui;
use update::{Check, Settings};

/// Default GitHub repository, overridable with `FONKETH_REPO`
const DEFAULT_REPO: &str = "Arvmor/fonketh";

/// Fonketh install wizard and launcher
///
/// Without a command it installs (or upgrades) the game interactively.
#[derive(Parser, Debug)]
#[command(name = "fonketh-installer", version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    install: InstallArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Install or upgrade the game (the default)
    Install(InstallArgs),
    /// Bring the game up to date, then start it
    Launch {
        /// Start the installed version without checking for updates
        #[arg(long)]
        no_update: bool,
        /// Arguments passed through to the game
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        game_args: Vec<String>,
    },
    /// Update the game to the latest release
    Update {
        /// Only report whether an update is available
        #[arg(long)]
        check: bool,
        /// Turn automatic updates on launch on or off
        #[arg(long, value_name = "on|off")]
        auto: Option<String>,
    },
    /// Remove the game (keeps your private key unless --purge)
    Uninstall {
        /// Also delete the private key
        #[arg(long)]
        purge: bool,
        /// Do not ask for confirmation
        #[arg(short = 'y', long, env = "FONKETH_YES")]
        yes: bool,
    },
}

#[derive(Args, Debug, Default)]
struct InstallArgs {
    /// Accept all defaults, never prompt
    #[arg(short = 'y', long, env = "FONKETH_YES")]
    yes: bool,

    /// Install a specific release (e.g. v0.2.0) instead of the latest
    #[arg(long, env = "FONKETH_VERSION", value_name = "TAG")]
    version: Option<String>,

    /// Install location [default: ~/.fonketh or %LOCALAPPDATA%\Programs\Fonketh]
    #[arg(long, value_name = "PATH")]
    dir: Option<PathBuf>,

    /// Where the `fonketh` command goes (macOS/Linux) [default: ~/.local/bin]
    #[arg(long, value_name = "PATH")]
    bin_dir: Option<PathBuf>,

    /// Install from a downloaded release archive instead of GitHub
    #[arg(long, value_name = "FILE")]
    archive: Option<PathBuf>,

    /// Do not touch PATH or shell startup files
    #[arg(long, env = "FONKETH_NO_MODIFY_PATH")]
    no_modify_path: bool,

    /// Do not update the game automatically on launch
    #[arg(long)]
    no_auto_update: bool,
}

fn main() {
    // Launched by double-click: keep the window open at the end so the result can be read
    let double_clicked = std::env::args().len() == 1;
    let cli = Cli::parse();

    let (ui, outcome) = match cli.command {
        None => {
            let ui = Ui::new(cli.install.yes);
            (ui, install(&cli.install, &ui))
        }
        Some(Command::Install(args)) => {
            let ui = Ui::new(args.yes);
            (ui, install(&args, &ui))
        }
        Some(Command::Launch {
            no_update,
            game_args,
        }) => (Ui::new(true), launch(no_update, &game_args)),
        Some(Command::Update { check, auto }) => (Ui::new(true), update(check, auto.as_deref())),
        Some(Command::Uninstall { purge, yes }) => {
            let ui = Ui::new(yes);
            (ui, uninstall(&ui, purge))
        }
    };

    if let Err(e) = &outcome {
        eprintln!("\n{} {e:#}", style("error:").red().bold());
    }
    if double_clicked {
        ui.wait_for_enter();
    }
    if outcome.is_err() {
        std::process::exit(1);
    }
}

fn repo() -> String {
    std::env::var("FONKETH_REPO").unwrap_or_else(|_| DEFAULT_REPO.to_string())
}

/// Home of an existing installation: the directory this launcher sits in, else the default
fn installed_layout() -> Result<Layout> {
    let mut layout = Layout::default_paths()?;
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("bin").is_dir()
        && dir.join("VERSION").is_file()
    {
        layout.home = dir.to_path_buf();
    }
    // The `fonketh` command may live somewhere other than the default bin dir
    if let Some(command) = Settings::load(&layout).command
        && let Some(dir) = command.parent()
    {
        layout.bin_dir = dir.to_path_buf();
    }
    layout.absolute()
}

fn install(args: &InstallArgs, ui: &Ui) -> Result<()> {
    let repo = repo();
    let mut layout = Layout::default_paths()?;
    if let Some(dir) = &args.dir {
        layout.home = dir.clone();
    }
    if let Some(dir) = &args.bin_dir {
        layout.bin_dir = dir.clone();
    }
    Ui::banner(&repo);

    Ui::step("Step 1/6  Checking your system");
    Ui::ok(format!("Detected {} ({TARGET})", os_name()));

    Ui::step("Step 2/6  Picking a release");
    let github = GitHub::new(&repo)?;
    let version = match (&args.archive, &args.version) {
        (_, Some(v)) => release::normalize_version(v),
        (Some(path), None) => {
            release::version_from_archive_name(path).unwrap_or_else(|| "local".to_string())
        }
        (None, None) => github.latest_version()?,
    };
    if args.archive.is_some() {
        Ui::ok(format!("Release {version} (local archive)"));
    } else {
        Ui::ok(format!("Release {version}"));
        if layout.installed_version().as_deref() == Some(&version) {
            Ui::ok(format!("Already installed at {}", layout.home.display()));
            if !ui.confirm("Reinstall anyway?", false)? {
                return Ok(());
            }
        }
    }

    Ui::step("Step 3/6  Choosing where to install");
    layout.home = PathBuf::from(ui.ask("Install directory", &layout.home.display().to_string())?);
    if !cfg!(windows) {
        layout.bin_dir =
            PathBuf::from(ui.ask("Launcher directory", &layout.bin_dir.display().to_string())?);
    }
    let layout = layout.absolute()?;
    if ui.interactive {
        if !ui.confirm(
            &format!("Install Fonketh {version} to {}?", layout.home.display()),
            true,
        )? {
            Ui::say("Aborted.");
            return Ok(());
        }
    } else {
        Ui::ok(format!("Installing to {}", layout.home.display()));
    }

    Ui::step("Step 4/6  Downloading and installing");
    let workdir = tempfile::Builder::new()
        .prefix("fonketh-install-")
        .tempdir()
        .context("creating a temporary directory")?;
    let (archive, source) =
        release::obtain_archive(&github, &version, args.archive.as_deref(), workdir.path())?;
    if matches!(source, Source::Local) {
        Ui::ok(format!("Using local archive {}", archive.display()));
    }
    layout.install_files(&archive, &version, workdir.path())?;
    // This installer becomes the launcher (unless the archive shipped a newer one)
    if !layout.launcher_bin().is_file() {
        layout.install_launcher_bin(&std::env::current_exe()?)?;
    }
    layout.install_launcher()?;
    platform::add_to_path(&layout, ui, !args.no_modify_path)?;
    platform::create_shortcuts(&layout, ui)?;

    Ui::step("Step 5/6  Updates");
    Ui::say("The launcher can check GitHub and update the game every time you start it.");
    let auto_update = if args.no_auto_update {
        false
    } else {
        ui.confirm("Keep the game up to date automatically?", true)?
    };
    Settings {
        auto_update,
        command: Some(layout.launcher()),
    }
    .save(&layout)?;
    if auto_update {
        Ui::ok("Automatic updates on");
    } else {
        Ui::ok("Automatic updates off. Update with: fonketh-launcher update");
    }

    Ui::step("Step 6/6  Miner key");
    key::setup(&layout, ui)?;

    summary(&layout, ui, &version)
}

fn summary(layout: &Layout, ui: &Ui, version: &str) -> Result<()> {
    Ui::step(&format!("Fonketh {version} is installed"));
    Ui::say(format!("Game files : {}", layout.home.display()));
    Ui::say(format!("Launcher   : {}", layout.launcher().display()));
    Ui::say(format!("Miner key  : {}", layout.key_file().display()));
    eprintln!();
    Ui::say(format!(
        "Run {} to play. Peers find you faster if UDP port 7331 is open.",
        style("fonketh").bold()
    ));
    for hint in platform::hints() {
        Ui::note(hint);
    }
    Ui::say(format!(
        "Uninstall with: {} uninstall",
        layout.launcher_bin().display()
    ));

    if ui.confirm("Launch Fonketh now?", false)? {
        std::process::Command::new(layout.game_binary())
            .current_dir(&layout.home)
            .spawn()
            .context("launching the game")?;
        Ui::ok("Launched");
    }
    Ok(())
}

/// `fonketh-launcher launch`: update when allowed, then hand over to the game
fn launch(no_update: bool, game_args: &[String]) -> Result<()> {
    let layout = installed_layout()?;
    let _ = std::fs::remove_file(layout.stale_launcher_bin());
    anyhow::ensure!(
        layout.game_binary().is_file(),
        "the game is not installed at {}, run the installer first",
        layout.home.display()
    );

    if !no_update {
        let settings = Settings::load(&layout);
        // Never keep the player from playing: any failure here is reported and ignored
        match check_and_apply(&layout, settings.auto_update_enabled()) {
            Ok(()) => {}
            Err(e) => Ui::warn(format!("Update check skipped: {e:#}")),
        }
    }

    let mut command = std::process::Command::new(layout.game_binary());
    command.args(game_args).current_dir(&layout.home);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec()).context("starting the game")
    }
    #[cfg(not(unix))]
    {
        command.spawn().context("starting the game")?;
        Ok(())
    }
}

fn check_and_apply(layout: &Layout, allowed: bool) -> Result<()> {
    let github = GitHub::new(&repo())?;
    match update::check(&github, layout)? {
        Check::UpToDate(_) => Ok(()),
        Check::Available { installed, latest } if allowed => {
            Ui::say(format!("Updating Fonketh {installed} -> {latest}"));
            update::apply(&github, layout, &latest)
        }
        Check::Available { latest, .. } => {
            Ui::note(format!(
                "Fonketh {latest} is available. Update with: {} update",
                layout.launcher_bin().display()
            ));
            Ok(())
        }
    }
}

/// `fonketh-launcher update`
fn update(check_only: bool, auto: Option<&str>) -> Result<()> {
    let layout = installed_layout()?;
    if let Some(value) = auto {
        let auto_update = match value.to_ascii_lowercase().as_str() {
            "on" | "true" | "1" => true,
            "off" | "false" | "0" => false,
            other => anyhow::bail!("--auto takes on or off, not '{other}'"),
        };
        let mut settings = Settings::load(&layout);
        settings.auto_update = auto_update;
        settings.save(&layout)?;
        Ui::ok(format!(
            "Automatic updates {}",
            if auto_update { "on" } else { "off" }
        ));
        if check_only || !auto_update {
            return Ok(());
        }
    }

    let github = GitHub::new(&repo())?;
    match update::check(&github, &layout)? {
        Check::UpToDate(version) => Ui::ok(format!("Fonketh {version} is up to date")),
        Check::Available { installed, latest } if check_only => Ui::say(format!(
            "Fonketh {latest} is available (installed: {installed})"
        )),
        Check::Available { installed, latest } => {
            Ui::say(format!("Updating Fonketh {installed} -> {latest}"));
            update::apply(&github, &layout, &latest)?;
        }
    }
    Ok(())
}

fn uninstall(ui: &Ui, purge: bool) -> Result<()> {
    let layout = installed_layout()?;
    Ui::banner(&repo());
    Ui::step("Uninstalling Fonketh");
    anyhow::ensure!(
        layout.home.exists() || layout.launcher().exists(),
        "nothing to uninstall at {}",
        layout.home.display()
    );
    if !ui.confirm(
        &format!(
            "Remove the game files in {} and the launcher?",
            layout.home.display()
        ),
        true,
    )? {
        return Ok(());
    }
    let key_exists = layout.key_file().is_file();
    let purge = purge
        || (key_exists
            && ui.confirm(
                &format!(
                    "Also delete the miner key {}? This cannot be undone",
                    layout.key_file().display()
                ),
                false,
            )?);
    layout.uninstall(purge)?;
    Ui::ok("Game files removed");
    if key_exists {
        if purge {
            Ui::ok("Miner key deleted");
        } else {
            Ui::note(format!("Kept {}", layout.key_file().display()));
        }
    }
    Ok(())
}

fn os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macOS",
        "windows" => "Windows",
        "linux" => "Linux",
        other => other,
    }
}
