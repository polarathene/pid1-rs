#[cfg(unix)]
mod cli;

fn main() {
  #[cfg(unix)]
  cli::Pid1App::from_cli().run();

  #[cfg(not(unix))]
  compile_error!("`pid1` is only compatible with Unix-like operating systems.");
}
