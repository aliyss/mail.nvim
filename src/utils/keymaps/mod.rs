#[must_use]
pub fn create_localized_keymap(
    cmd: &str,
    start_line: usize,
    end_line: usize,
    err_msg: &str,
) -> String {
    let cmd = format!(
        "<cmd>lua if vim.fn.line('.') >= {start_line} and vim.fn.line('.') <= {end_line} then \
     vim.cmd(\"{cmd}\") \
     else \
     print('{err_msg}') \
     end<CR>",
    );
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keymap_keeps_require_quotes_intact() {
        let keymap = create_localized_keymap(
            "lua require('mail_nvim').ui_enter()",
            5,
            10,
            "outside the list",
        );

        assert!(
            keymap.contains("vim.cmd(\"lua require('mail_nvim').ui_enter()\")"),
            "nested single quotes must not terminate the outer Lua string: {keymap}"
        );
    }
}
