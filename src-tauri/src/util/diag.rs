//! Output that cannot crash the caller.
//!
//! `println!` and `eprintln!` panic when the write fails: a closed terminal, a
//! closed pipe, or a GUI process whose console went away. That is how a routine
//! progress line became a recorded crash —
//! `failed printing to stderr: The pipe is being closed. (os error 232)` — and
//! the Health page then reported it for the rest of the session.
//!
//! Background threads (crawl, watcher, embedding) must not die because nobody
//! is listening to their output, so they go through these macros instead. The
//! argument syntax is identical to the std macros.

/// Writes a line to stdout, ignoring a broken pipe.
#[macro_export]
macro_rules! say {
    ($($arg:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::writeln!(::std::io::stdout(), $($arg)*);
    }};
}

/// Writes a line to stderr, ignoring a broken pipe.
#[macro_export]
macro_rules! say_err {
    ($($arg:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::writeln!(::std::io::stderr(), $($arg)*);
    }};
}
