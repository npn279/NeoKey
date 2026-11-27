#[cfg(target_os = "macos")]
mod macos;

fn main() -> std::process::ExitCode {
    #[cfg(target_os = "macos")]
    let result = macos::run();
    #[cfg(not(target_os = "macos"))]
    let result = Err("NeoKey requires macOS");

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            std::process::ExitCode::FAILURE
        }
    }
}
