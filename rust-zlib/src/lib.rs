extern crate libc;


use libc::{c_void, size_t, c_int};

#[link(name = "miniz", kind = "static")]
extern {
    /// Raw miniz compression function.
    fn tdefl_compress_mem_to_heap(psrc_buf: *const c_void,
                                  src_buf_len: size_t,
                                  pout_len: *mut size_t,
                                  flags: c_int)
                                  -> *mut c_void;

    /// Raw miniz decompression function.
    fn tinfl_decompress_mem_to_heap(psrc_buf: *const c_void,
                                    src_buf_len: size_t,
                                    pout_len: *mut size_t,
                                    flags: c_int)
                                    -> *mut c_void;
}





#[test]
fn it_works() {
}
