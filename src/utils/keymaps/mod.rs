#[must_use]
pub fn create_localized_keymap(
    cmd: &str,
    start_line: usize,
    end_line: usize,
    err_msg: &str,
) -> String {
    let cmd = format!(
        "<cmd>lua if vim.fn.line('.') >= {start_line} and vim.fn.line('.') <= {end_line} then \
     vim.cmd('{cmd}') \
     else \
     print('{err_msg}') \
     end<CR>",
    );
    cmd
}
