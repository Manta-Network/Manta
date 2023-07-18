/// Returns a reference to the elements of `$slice` as an array, verifying that
/// the slice is of length `$len`.
///
/// source: https://github.com/rust-lang/rfcs/issues/1833
#[macro_export]
macro_rules! slice_as_array_ref {
    ($slice:expr, $len:expr) => {{
        use crate::types::errors::Kind;

        fn slice_as_array_ref<T>(slice: &[T]) -> Result<&[T; $len], Kind> {
            if slice.len() != $len {
                return Err(Kind::InvalidDataLength {
                    expected: $len,
                    current: slice.len(),
                });
            }
            Ok(unsafe { &*(slice.as_ptr() as *const [T; $len]) })
        }
        slice_as_array_ref($slice)
    }};
}
