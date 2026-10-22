#![no_main]
use dibibest_core::{sql::SqlCodec, Codec};
use libfuzzer_sys::fuzz_target;

// Contract: parse never panics on any UTF-8 input; Err is fine.
fuzz_target!(|src: &str| {
    let _ = SqlCodec::default().parse(src);
});
