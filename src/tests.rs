//! Integration tests that run inside a real (headless) Neovim.
//!
//! Each `#[nvim_oxi::test]` function runs in its own Neovim instance: the
//! macro expands into a normal Rust `#[test]` that spawns `nvim --headless`,
//! loads this crate as a plugin and runs the function body against the real
//! Neovim API. See `nvim-oxi`'s `tests` module for details.

pub mod completion;
pub mod compose;
pub mod confirm;
pub mod email;
pub mod fake;
pub mod loading;
pub mod view;

// A smoke test: the harness itself works.
#[nvim_oxi::test]
fn harness_smoke_test() {
    assert_eq!(nvim_oxi::api::get_current_buf().line_count().unwrap(), 1);
}

mod panic_safety {
    use nvim_oxi::api::types::CommandArgs;

    use crate::commands::{Name, UserCommand};

    /// A command whose callback panics on purpose, to verify that panics are
    /// caught and Neovim survives.
    struct PanicCommand;

    impl UserCommand for PanicCommand {
        const NAME: Name = Name::new("TestPanic");
        const DESCRIPTION: &'static str = "panics on purpose";

        fn callback(_: CommandArgs) {
            panic!("intentional panic for testing");
        }
    }

    #[nvim_oxi::test]
    fn panicking_command_does_not_crash_neovim() {
        PanicCommand::register().expect("failed to register the panicking command");
        nvim_oxi::api::command("TestPanic").expect("failed to run the panicking command");

        // If the panic had unwound across the FFI boundary, the whole
        // (headless) Neovim process would have been corrupted or crashed and
        // this test would fail instead of reaching the end.
    }

    #[nvim_oxi::test]
    fn early_returning_command_does_not_error() {
        /// A command that reports a user-facing error via `bail!`.
        struct BailCommand;

        impl UserCommand for BailCommand {
            const NAME: Name = Name::new("TestBail");
            const DESCRIPTION: &'static str = "bails on purpose";

            fn callback(_: CommandArgs) {
                crate::bail!("expected error: cannot do the thing");
            }
        }

        BailCommand::register().expect("failed to register the bailing command");
        nvim_oxi::api::command("TestBail").expect("failed to run the bailing command");
    }
}
