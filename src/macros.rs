/// A convenience macro for early returns in user commands.
#[macro_export]
macro_rules! bail {
    ($message:literal $(,)?) => {
        // An extra block so the macro can be used as a one-liner in a match statement.
        {
            ::nvim_oxi::print!($message);
            return;
        }
    };
    ($fmt:literal, $($arg:tt)*) => {
        {
            ::nvim_oxi::print!($fmt, $($arg)*);
            return;
        }
    };
}
