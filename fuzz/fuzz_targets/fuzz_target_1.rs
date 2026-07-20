#![no_main]

use libfuzzer_sys::fuzz_target;
use remitflow_contract::*;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
});
