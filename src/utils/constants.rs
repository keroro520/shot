pub const CELLBASE_MATURITY: u64 = 7200;
pub const MIN_OUTPUT_CAPACITY: u64 = 61_0000_0000;
pub const MIN_FEE_RATE: u64 = 1000; // shannons/KB
pub const MIN_INPUT_CAPACITY: u64 = MIN_OUTPUT_CAPACITY + MIN_OUTPUT_CAPACITY / MIN_FEE_RATE + 1;
