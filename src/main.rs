use std::process::ExitCode;

fn main() -> ExitCode {
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    match std::panic::catch_unwind(|| princi::run(std::env::args_os().skip(1))) {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(error)) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
        Err(_) => {
            eprintln!(
                "{}",
                princi::diagnostics::Diagnostic::coded(
                    princi::diagnostics::DiagnosticCode::InternalCompiler,
                    "unexpected compiler failure; please report this issue",
                )
            );
            ExitCode::FAILURE
        }
    }
}
