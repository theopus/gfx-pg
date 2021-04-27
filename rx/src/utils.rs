use std::mem::size_of;

#[allow(dead_code)]
pub fn cast_slice<T, U>(ts: &[T]) -> Option<&[U]> {
    use core::mem::align_of;
    // Handle ZST (this all const folds)
    if size_of::<T>() == 0 || size_of::<U>() == 0 {
        if size_of::<T>() == size_of::<U>() {
            unsafe {
                return Some(core::slice::from_raw_parts(
                    ts.as_ptr() as *const U,
                    ts.len(),
                ));
            }
        } else {
            return None;
        }
    }
    // Handle alignments (this const folds)
    if align_of::<U>() > align_of::<T>() {
        // possible mis-alignment at the new type (this is a real runtime check)
        if (ts.as_ptr() as usize) % align_of::<U>() != 0 {
            return None;
        }
    }
    if size_of::<T>() == size_of::<U>() {
        // same size, so we direct cast, keeping the old length
        unsafe {
            Some(core::slice::from_raw_parts(
                ts.as_ptr() as *const U,
                ts.len(),
            ))
        }
    } else {
        // we might have slop, which would cause us to fail
        let byte_size = size_of::<T>() * ts.len();
        let (new_count, new_overflow) = (byte_size / size_of::<U>(), byte_size % size_of::<U>());
        if new_overflow > 0 {
            return None;
        } else {
            unsafe {
                Some(core::slice::from_raw_parts(
                    ts.as_ptr() as *const U,
                    new_count,
                ))
            }
        }
    }
}
