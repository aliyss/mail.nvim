//! Applies syntax highlighting to mail buffers.
//!
//! The syntax files are applied explicitly after a component has been
//! rendered into a buffer instead of relying on a `FileType` autocommand,
//! which guarantees the highlighting (and the metadata folding) is computed
//! against the final buffer content.

use nvim_oxi::api::opts::{ExecOpts, OptionOpts};
use nvim_oxi::api::{self, Buffer};

const TABLE_SYNTAX: &str = include_str!("../syntax/mail-table.vim");
const FILE_SYNTAX: &str = include_str!("../syntax/mail-file.vim");
const DRAWER_SYNTAX: &str = include_str!("../syntax/mail-drawer.vim");
const COMPOSE_SYNTAX: &str = include_str!("../syntax/mail-compose.vim");

/// Applies the syntax file matching the buffer's filetype, if any.
///
/// # Errors
///
/// Returns an error if the buffer's filetype cannot be read or the syntax
/// code fails to execute.
pub fn apply(buffer: &Buffer) -> anyhow::Result<()> {
    let opts = OptionOpts::builder().buf(buffer.clone()).build();
    let filetype = api::get_option_value::<String>("filetype", &opts)?;

    let code = match filetype.as_str() {
        "mail-table" => TABLE_SYNTAX,
        "mail-file" => FILE_SYNTAX,
        "mail-drawer" => DRAWER_SYNTAX,
        "mail-compose" => COMPOSE_SYNTAX,
        _ => return Ok(()),
    };

    let exec_opts = ExecOpts::builder().output(false).build();
    api::exec2(code, &exec_opts)?;

    Ok(())
}
