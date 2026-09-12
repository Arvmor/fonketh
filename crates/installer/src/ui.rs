//! Terminal output and prompts for the wizard
//!
//! Every prompt has a default so a non-interactive run (`--yes`, or no
//! terminal attached) makes the same choices a user pressing Enter would.

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use std::io::IsTerminal;

/// How the wizard talks to the user
#[derive(Clone, Copy)]
pub struct Ui {
    /// Prompts are shown; otherwise defaults are taken silently
    pub interactive: bool,
}

impl Ui {
    /// Interactive unless `--yes` was given or there is no terminal to ask on
    pub fn new(assume_yes: bool) -> Self {
        let interactive =
            !assume_yes && std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        Self { interactive }
    }

    pub fn banner(repo: &str) {
        let art = r#"
   ___          _        _   _
  | __|__ _ __ | |__ ___| |_| |__
  | _/ _ \ ' \ | / // -_)  _| ' \
  |_|\___/_||_||_\_\\___|\__|_||_|"#;
        eprintln!("{}", style(art).cyan());
        eprintln!("  P2P mining pool, gamified. https://github.com/{repo}\n");
    }

    pub fn step(text: &str) {
        eprintln!("\n{} {}", style("==>").cyan(), style(text).bold());
    }

    pub fn ok(text: impl AsRef<str>) {
        eprintln!("  {} {}", style("✓").green(), text.as_ref());
    }

    pub fn note(text: impl AsRef<str>) {
        eprintln!("  {}", style(text.as_ref()).dim());
    }

    pub fn warn(text: impl AsRef<str>) {
        eprintln!("  {} {}", style("!").yellow(), text.as_ref());
    }

    pub fn say(text: impl AsRef<str>) {
        eprintln!("  {}", text.as_ref());
    }

    /// Free text with a default
    pub fn ask(&self, prompt: &str, default: &str) -> Result<String> {
        if !self.interactive {
            return Ok(default.to_string());
        }
        let answer: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("  {prompt}"))
            .default(default.to_string())
            .interact_text()?;
        Ok(answer.trim().to_string())
    }

    /// Yes/no question
    pub fn confirm(&self, prompt: &str, default: bool) -> Result<bool> {
        if !self.interactive {
            return Ok(default);
        }
        Ok(Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("  {prompt}"))
            .default(default)
            .interact()?)
    }

    /// Pick one of several options, returns the index
    pub fn choose(&self, prompt: &str, options: &[&str], default: usize) -> Result<usize> {
        if !self.interactive {
            return Ok(default);
        }
        Ok(Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("  {prompt}"))
            .items(options)
            .default(default)
            .interact()?)
    }

    /// Hidden input, only available interactively
    pub fn secret(&self, prompt: &str) -> Result<String> {
        anyhow::ensure!(
            self.interactive,
            "a private key can only be imported interactively"
        );
        Ok(Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("  {prompt}"))
            .interact()?)
    }

    /// Keeps a double-clicked console window open until the user has read it
    pub fn wait_for_enter(&self) {
        if self.interactive {
            eprint!("\n  Press Enter to close this window.");
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
        }
    }
}
