/// utility macro to quickly format cents
#[macro_export]
macro_rules! format_cents {
    ($num:expr) => {
        TokenBalance::new($num, 6).humanize_float_truncate_to_dp(2)
    };
}
