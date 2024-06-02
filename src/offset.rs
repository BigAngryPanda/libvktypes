#[inline]
pub(crate) fn padding_bytes(last: u64, alignment: u64) -> u64 {
    ((last % alignment != 0) as u64)*(alignment - last % alignment)
}

#[inline]
pub(crate) fn full_size(requested_size: u64, alignment: u64) -> u64 {
    requested_size + padding_bytes(requested_size, alignment)
}
