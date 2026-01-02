use crate::cli::{CompletionArgs, CompletionShell};
use clap::CommandFactory;

pub fn run(args: CompletionArgs) -> i32 {
    let mut cmd = crate::cli::App::command();
    match args.shell {
        CompletionShell::Nushell => {
            clap_complete::generate(
                clap_complete_nushell::Nushell,
                &mut cmd,
                "jwt-tester",
                &mut std::io::stdout(),
            );
        }
        other => {
            let shell = match other {
                CompletionShell::Bash => clap_complete::Shell::Bash,
                CompletionShell::Zsh => clap_complete::Shell::Zsh,
                CompletionShell::Fish => clap_complete::Shell::Fish,
                CompletionShell::Powershell => clap_complete::Shell::PowerShell,
                CompletionShell::Elvish => clap_complete::Shell::Elvish,
                CompletionShell::Nushell => unreachable!("handled above"),
            };
            clap_complete::generate(shell, &mut cmd, "jwt-tester", &mut std::io::stdout());
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_run_for_all_shells() {
        let shells = [
            CompletionShell::Bash,
            CompletionShell::Zsh,
            CompletionShell::Fish,
            CompletionShell::Powershell,
            CompletionShell::Elvish,
            CompletionShell::Nushell,
        ];
        for shell in shells {
            let code = run(CompletionArgs { shell });
            assert_eq!(code, 0);
        }
    }
}
