#![warn(
    clippy::pedantic,
    clippy::multiple_crate_versions,
    clippy::cognitive_complexity,
    clippy::missing_const_for_fn,
    clippy::needless_borrow,
    clippy::redundant_pub_crate,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_let_if_seq,
    rust_2018_idioms,
    future_incompatible
)]
#![allow(clippy::missing_errors_doc, clippy::match_bool, clippy::map_err_ignore)]

pub mod cpu;
pub mod memory;
pub mod registers;
pub mod utils;
