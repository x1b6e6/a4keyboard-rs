use std::mem::size_of_val;
use std::slice;

/// SAFETY: must work only for simple types
pub unsafe trait AsBytes {
    fn as_bytes(&self) -> &[u8];
    fn as_bytes_mut(&mut self) -> &mut [u8];
}

/// SAFETY: `T` is simple type because it requires `Copy` trait
unsafe impl<T: Copy> AsBytes for T {
    fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const Self as *const u8, size_of_val(self)) }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self as *mut Self as *mut u8, size_of_val(self)) }
    }
}

/// SAFETY: buffer of `[T]` is simple type because `T` requires `Copy` trait
unsafe impl<T: Copy> AsBytes for [T] {
    fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr() as *const T as *const u8, size_of_val(self)) }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.as_mut_ptr() as *mut T as *mut u8, size_of_val(self))
        }
    }
}

macro_rules! startup {
    ($($tt:tt)*) => {
        #[link_section = ".init_array"]
        #[used]
        static mut STARTUP: extern "C" fn() = {
            extern "C" fn f()  {
                $($tt)*
            }
            f
        };
    };
}

pub(crate) use startup;
