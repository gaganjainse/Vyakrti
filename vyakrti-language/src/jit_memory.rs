use std::ptr;

pub struct ExecutableMemory {
    ptr: *mut u8,
    size: usize,
}

impl ExecutableMemory {
    pub fn allocate(size: usize) -> Self {
        let page_size = 4096;
        let aligned_size = (size + page_size - 1) & !(page_size - 1);
        unsafe {
            #[cfg(unix)]
            {
                let mut page_ptr: *mut std::ffi::c_void = ptr::null_mut();
                let res = libc::posix_memalign(&mut page_ptr, page_size, aligned_size);
                if res != 0 { panic!("System Allocation Fault: Unable to map aligned pages via kernel."); }
                libc::mprotect(page_ptr, aligned_size, libc::PROT_READ | libc::PROT_WRITE);
                ExecutableMemory { ptr: page_ptr as *mut u8, size: aligned_size }
            }
            #[cfg(windows)]
            {
                let raw = winapi::um::memoryapi::VirtualAlloc(
                    ptr::null_mut(), aligned_size, winapi::um::winnt::MEM_COMMIT, winapi::um::winnt::PAGE_READWRITE
                );
                if raw.is_null() {
                    panic!("System Allocation Fault: Unable to reserve executable virtual memory pages.");
                }
                ExecutableMemory { ptr: raw as *mut u8, size: aligned_size }
            }
            #[cfg(not(any(unix, windows)))]
            {
                panic!("JIT Memory: Unsupported target platform for executable memory allocation.");
            }
        }
    }

    pub fn write_bytes(&mut self, machine_code: &[u8]) {
        assert!(machine_code.len() <= self.size, "JIT Core Crash: Byte sequences out of bounds of aligned page context memory.");
        unsafe { ptr::copy_nonoverlapping(machine_code.as_ptr(), self.ptr, machine_code.len()); }
    }

    pub fn seal_and_protect_page(&self) {
        unsafe {
            #[cfg(unix)]
            { libc::mprotect(self.ptr as *mut std::ffi::c_void, self.size, libc::PROT_READ | libc::PROT_EXEC); }
            #[cfg(windows)]
            {
                let mut old = 0;
                let res = winapi::um::memoryapi::VirtualProtect(
                    self.ptr as *mut winapi::ctypes::c_void, self.size, winapi::um::winnt::PAGE_EXECUTE_READ, &mut old
                );
                if res == 0 {
                    panic!("JIT Memory: Failed to seal executable page with read-execute protection.");
                }
            }
        }
    }

    pub fn execute_fn(&self, input: i64) -> i64 {
        unsafe {
            let bare_metal_call: extern "C" fn(i64) -> i64 = std::mem::transmute(self.ptr);
            bare_metal_call(input)
        }
    }
}

impl Drop for ExecutableMemory {
    fn drop(&mut self) {
        unsafe {
            #[cfg(unix)] { libc::free(self.ptr as *mut std::ffi::c_void); }
            #[cfg(windows)] {
                winapi::um::memoryapi::VirtualFree(self.ptr as *mut winapi::ctypes::c_void, 0, winapi::um::winnt::MEM_RELEASE);
            }
        }
    }
}
