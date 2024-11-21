#[macro_export]
macro_rules! printv {
    ( $($x:expr),* ) => {{
        dbg!($($x.clone()),*);
        println!();
    }}
}

#[macro_export]
macro_rules! show_input {
    ($file:expr) => {{
        println!("\r\nInput: --------------------------------------\r\n");
        $file.print();
        println!("\r\n----------------------------------------------\r\n");
    }}
}

#[macro_export]
macro_rules! show_result {
    ($file:expr) => {{
        println!("\r\nResult: --------------------------------------\r\n");
        $file.print();
        println!("\r\n----------------------------------------------\r\n");
    }}
}