#[cfg(target_family = "unix")]
use pid1::Pid1Settings;
#[cfg(target_family = "unix")]
use signal_hook::{
    consts::{SIGCHLD, SIGINT, SIGTERM},
    iterator::Signals,
};
#[cfg(target_family = "unix")]
use std::os::unix::process::CommandExt;
#[cfg(target_family = "unix")]
use std::time::Duration;
use std::{str::FromStr, ffi::OsString, path::PathBuf};

#[derive(clap::Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub(crate) struct Pid1App {
    /// Specify working directory
    #[arg(short, long, value_name = "DIR")]
    pub(crate) workdir: Option<PathBuf>,

    /// Grace period for stopping before escalating to SIGKILL
    #[arg(short, long, value_name = "SECONDS", default_value_t = 2)]
    pub(crate) timeout: u8,

    /// Turn on verbose output
    #[arg(short, long)]
    pub(crate) verbose: bool,

    /// Override environment variables. Can specify multiple times.
    #[arg(short, long, value_name = "KEY=VALUE")]
    pub(crate) env: Vec<KeyValue>,

    /// Run command with user ID
    #[arg(short, long, value_name = "USER ID")]
    pub(crate) user_id: Option<u32>,

    /// Run command with group ID
    #[arg(short, long, value_name = "GROUP ID")]
    pub(crate) group_id: Option<u32>,

    /// Process to run
    #[arg(trailing_var_arg = true)]
    pub(crate) command: String,

    /// Arguments to that process
    pub(crate) args: Vec<String>,
}

impl Pid1App {
    #[cfg(target_family = "unix")]
    pub(crate) fn run(self) -> ! {
        let mut child = std::process::Command::new(&self.command);
        let child = child.args(&self.args[..]);
        if let Some(workdir) = &self.workdir {
            child.current_dir(workdir);
        }
        if let Some(user_id) = &self.user_id {
            child.uid(*user_id);
        }
        if let Some(group_id) = &self.group_id {
            child.gid(*group_id);
        }
        for KeyValue(key, value) in &self.env {
            child.env(key, value);
        }
        let pid = std::process::id();
        if pid != 1 {
            let status = child.exec();
            eprintln!("execvp failed with: {status:?}");

            std::process::exit(1);
        } else {
            // Install signal handlers before launching child process
            let signals = Signals::new([SIGTERM, SIGINT, SIGCHLD]).unwrap();
            let child = child.spawn();
            let child = match child {
                Ok(child) => child,
                Err(err) => {
                    eprintln!("pid1: {} spawn failed. Got error: {err}", self.command);
                    std::process::exit(1);
                }
            };

            Pid1Settings::new()
                .enable_log(self.verbose)
                .timeout(Duration::from_secs(self.timeout.into()))
                .pid1_handling(signals, child)
        }
    }

    #[cfg(target_family = "windows")]
    pub(crate) fn run(self) -> ! {
        eprintln!("pid1: Not supported on Windows");
        std::process::exit(1);
    }
}

/// A CLI argument parsed from a `KEY=VALUE` pair.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KeyValue(OsString, OsString);

impl FromStr for KeyValue {
    type Err = String;

    /// Parses a CLI flag value into a [`KeyValue`] type.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s.split_once('=')
          .ok_or_else(|| format!("invalid `KEY=VALUE` pair: no `=` found in `{s}`"))?;

        Ok(KeyValue(key.into(), value.into()))
    }
}
