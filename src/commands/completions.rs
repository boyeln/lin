//! Shell completion generation commands.
//!
//! Commands for generating shell completion scripts for various shells.

use clap::Command;
use clap_complete::{Shell, generate};
use std::io;

/// Generate shell completion scripts.
///
/// Outputs completion script for the specified shell to stdout.
/// Users can redirect this output to their shell's completion directory.
///
/// # Arguments
///
/// * `shell` - The shell to generate completions for
/// * `cmd` - The clap Command to generate completions from
///
/// # Example
///
/// ```ignore
/// use clap::Command;
/// use lin::commands::completions::generate_completions;
/// use clap_complete::Shell;
///
/// let mut cmd = Command::new("lin");
/// generate_completions(Shell::Bash, &mut cmd);
/// ```
pub fn generate_completions(shell: Shell, cmd: &mut Command) {
    generate(shell, cmd, "lin", &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    fn build_test_command() -> Command {
        Command::new("lin")
            .subcommand(Command::new("issue").subcommand(Command::new("list")))
            .subcommand(Command::new("team").subcommand(Command::new("list")))
    }

    #[test]
    fn test_generate_bash_completions() {
        let mut cmd = build_test_command();
        let mut output = Vec::new();
        generate(Shell::Bash, &mut cmd, "lin", &mut output);

        let output_str = String::from_utf8(output).expect("Output should be valid UTF-8");
        assert!(
            output_str.contains("lin"),
            "Bash completion should contain command name"
        );
        assert!(
            output_str.contains("_lin"),
            "Bash completion should contain completion function"
        );
    }

    #[test]
    fn test_generate_zsh_completions() {
        let mut cmd = build_test_command();
        let mut output = Vec::new();
        generate(Shell::Zsh, &mut cmd, "lin", &mut output);

        let output_str = String::from_utf8(output).expect("Output should be valid UTF-8");
        assert!(
            output_str.contains("lin"),
            "Zsh completion should contain command name"
        );
        assert!(
            output_str.contains("#compdef"),
            "Zsh completion should contain compdef directive"
        );
    }

    #[test]
    fn test_generate_fish_completions() {
        let mut cmd = build_test_command();
        let mut output = Vec::new();
        generate(Shell::Fish, &mut cmd, "lin", &mut output);

        let output_str = String::from_utf8(output).expect("Output should be valid UTF-8");
        assert!(
            output_str.contains("lin"),
            "Fish completion should contain command name"
        );
        assert!(
            output_str.contains("complete"),
            "Fish completion should contain complete command"
        );
    }

    #[test]
    fn test_generate_powershell_completions() {
        let mut cmd = build_test_command();
        let mut output = Vec::new();
        generate(Shell::PowerShell, &mut cmd, "lin", &mut output);

        let output_str = String::from_utf8(output).expect("Output should be valid UTF-8");
        assert!(
            output_str.contains("lin"),
            "PowerShell completion should contain command name"
        );
        assert!(
            output_str.contains("Register-ArgumentCompleter"),
            "PowerShell completion should contain Register-ArgumentCompleter"
        );
    }

    #[test]
    fn test_generate_elvish_completions() {
        let mut cmd = build_test_command();
        let mut output = Vec::new();
        generate(Shell::Elvish, &mut cmd, "lin", &mut output);

        let output_str = String::from_utf8(output).expect("Output should be valid UTF-8");
        assert!(
            output_str.contains("lin"),
            "Elvish completion should contain command name"
        );
        assert!(
            output_str.contains("edit:completion"),
            "Elvish completion should contain edit:completion"
        );
    }

    #[test]
    fn test_completions_include_subcommands() {
        let mut cmd = build_test_command();
        let mut output = Vec::new();
        generate(Shell::Bash, &mut cmd, "lin", &mut output);

        let output_str = String::from_utf8(output).expect("Output should be valid UTF-8");
        assert!(
            output_str.contains("issue"),
            "Completion should include 'issue' subcommand"
        );
        assert!(
            output_str.contains("team"),
            "Completion should include 'team' subcommand"
        );
        assert!(
            output_str.contains("list"),
            "Completion should include 'list' subcommand"
        );
    }
}
