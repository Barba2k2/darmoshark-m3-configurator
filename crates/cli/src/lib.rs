//! `dms`, the command line front-end over the `darmoshark` crate.
//!
//! - `arguments`: the clap definition of every subcommand.
//! - `commands`: what each subcommand does, returning the text to print.

// Constants follow the project naming rule: camelCase, never SCREAMING_CASE.
#![allow(non_upper_case_globals)]

pub mod arguments;
pub mod commands;
