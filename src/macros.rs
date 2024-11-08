#[macro_export]
macro_rules! printv {
    ( $($x:expr),* ) => {{
        dbg!($($x.clone()),*);
        println!();
    }}
}
