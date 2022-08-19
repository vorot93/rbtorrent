use rbtorrent::ffi::version;
use std::ffi::CStr;

fn main() {
    unsafe {
        println!("libtorrent version: {:?}", CStr::from_ptr(version()));
    }
}
