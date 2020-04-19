#![no_main]
use libfuzzer_sys::fuzz_target;
use pact::types::Contract;

fuzz_target!(|data: &[u8]| {
    let _ = Contract::decode(&mut &data[..]);
});
