// #######################
//   Definições públicas
// #######################
macro_rules! log {
    (level $lvl:expr, color $col:ident, $msg:expr) => {
        {
            use colored::Colorize as _;
            println!("[{}] {}", $lvl.$col(), $msg);
        }
    };

    (level $lvl:expr, color $col:ident, $($arg:expr),*) => {
        {
            use colored::Colorize as _;
            let msg = format!($($arg),*);
            println!("[{}] {}", $lvl.$col(), msg);
        }
    };
}

macro_rules! info {
    ( $msg:literal ) => {
        log!(level "INFO", color blue, $msg);
    };

    ( $($arg:expr),* ) => {
        log!(level "INFO", color blue, $($arg),*);
    };
}

macro_rules! error {
    ( $msg:literal ) => {
        log!(level "ERROR", color red, $msg);
    };

    ( $($arg:expr),* ) => {
        log!(level "ERROR", color red, $($arg),*);
    };
}

pub(crate) use error;
pub(crate) use info;
pub(crate) use log;
