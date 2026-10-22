#![no_main]
use dibibest_core::{dbml::DbmlCodec, Codec};
use libfuzzer_sys::fuzz_target;

// Contract: parse never panics on any UTF-8 input; Err is fine.
fuzz_target!(|src: &str| {
    let _ = DbmlCodec.parse(src);
});
