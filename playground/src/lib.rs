use std::{cell::RefCell, slice};

thread_local! {
    static OUTPUT: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static DIAGNOSTICS: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static SUCCESS: RefCell<u32> = RefCell::new(0);
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, len);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_playground(ptr: *const u8, len: usize) -> u32 {
    let source = unsafe { slice::from_raw_parts(ptr, len) };
    let source = match std::str::from_utf8(source) {
        Ok(source) => source,
        Err(err) => {
            write_result("", &format!("input must be valid UTF-8: {err}"), 0);
            return 0;
        }
    };

    let result = tol_lang::driver::run_source(source, "playground.tol");
    let diagnostics = result.diagnostics.join("\n\n");
    let success = u32::from(result.success && result.diagnostics.is_empty());

    write_result(&result.output, &diagnostics, success);
    success
}

#[unsafe(no_mangle)]
pub extern "C" fn output_ptr() -> *const u8 {
    OUTPUT.with(|buf| buf.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn output_len() -> usize {
    OUTPUT.with(|buf| buf.borrow().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn diagnostics_ptr() -> *const u8 {
    DIAGNOSTICS.with(|buf| buf.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn diagnostics_len() -> usize {
    DIAGNOSTICS.with(|buf| buf.borrow().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn success() -> u32 {
    SUCCESS.with(|flag| *flag.borrow())
}

fn write_result(output: &str, diagnostics: &str, success: u32) {
    OUTPUT.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        buf.extend_from_slice(output.as_bytes());
    });

    DIAGNOSTICS.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        buf.extend_from_slice(diagnostics.as_bytes());
    });

    SUCCESS.with(|flag| {
        *flag.borrow_mut() = success;
    });
}
