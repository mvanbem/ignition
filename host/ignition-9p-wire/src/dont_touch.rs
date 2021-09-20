/// Provides the "don't touch" value for use in `wstat` requests.
///
/// A `wstat` request can avoid modifying some properties of the file by providing explicit "don't
/// touch" values in the stat data that is sent: zero-length strings for text values and the
/// maximum unsigned value of appropriate size for integral values.
pub trait DontTouch {
    /// Returns the "don't touch" value for this type.
    fn dont_touch() -> Self;
}
impl DontTouch for u8 {
    fn dont_touch() -> Self {
        !0
    }
}
impl DontTouch for u16 {
    fn dont_touch() -> Self {
        !0
    }
}
impl DontTouch for u32 {
    fn dont_touch() -> Self {
        !0
    }
}
impl DontTouch for u64 {
    fn dont_touch() -> Self {
        !0
    }
}
impl DontTouch for String {
    fn dont_touch() -> Self {
        String::new()
    }
}
