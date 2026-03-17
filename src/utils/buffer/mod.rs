use nvim_oxi::api::{
    self, Buffer, Window,
    opts::{OptionOpts, OptionScope},
};

pub mod metadata;
pub mod render;

pub fn get_real_width(win: &Window, buf: &Buffer) -> nvim_oxi::Result<u16> {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();

    // win.get_width() typically returns usize or i32 depending on oxi version.
    // We treat everything as i64 internally to handle subtractions safely.
    let win_width = i64::from(win.get_width()?);

    // 1. Account for Line Numbers
    let mut gutter_width: i64 = 0;
    let number: bool = api::get_option_value("number", &opts)?;
    let relativenumber: bool = api::get_option_value("relativenumber", &opts)?;

    if number || relativenumber {
        let numberwidth: i64 = api::get_option_value("numberwidth", &opts)?;
        let line_count = buf.line_count()?;
        // Use .ilog10() or len() to find digits; cast carefully
        let needed_digits = i64::try_from(line_count.to_string().len()).unwrap_or(1);
        gutter_width += std::cmp::max(numberwidth, needed_digits) + 1;
    }

    // 2. Account for Sign Column
    let signcolumn: String = api::get_option_value("signcolumn", &opts)?;
    if signcolumn != "no" {
        gutter_width += 2;
    }

    // 3. Account for Fold Column
    let foldcolumn: String = api::get_option_value("foldcolumn", &opts)?;
    if let Ok(fold_w) = foldcolumn.parse::<i64>() {
        gutter_width += fold_w;
    }

    // Final calculation: Subtract, ensure it's not negative, then try_into u16
    let final_width = (win_width - gutter_width).max(0);

    // try_from handles the "truncation" and "sign loss" lints explicitly
    Ok(u16::try_from(final_width).unwrap_or(80))
}
