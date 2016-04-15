
// log dummy macros
#[cfg(not(feature="logger"))]
macro_rules! debug (
    ($($arg:tt)*) => ()
);

#[cfg(not(feature="logger"))]
macro_rules! error (
    ($($arg:tt)*) => ()
);

#[cfg(not(feature="logger"))]
macro_rules! info (
    ($($arg:tt)*) => ()
);

#[cfg(not(feature="logger"))]
macro_rules! trace (
    ($($arg:tt)*) => ()
);

#[cfg(not(feature="logger"))]
macro_rules! warn (
    ($($arg:tt)*) => ()
);

#[cfg(not(feature="logger"))]
macro_rules! log_enabled (
    ($some:expr) => (false)
);
