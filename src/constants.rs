/// We use fixed-point arithmetic for prices to avoid floating-point issues
/// while maintaining precision. A factor of 1_000 gives us 3 decimal places.
/// E.g., a stored price of 12500 represents a display price of 12.500.
pub(crate) const PRICE_SCALE_FACTOR: u128 = 1_000;