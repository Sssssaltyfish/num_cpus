#[cfg(not(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "aix",
    feature = "axstd",
)))]
#[inline]
fn get_num_physical_cpus() -> usize {
    // Not implemented, fall back
    get_num_cpus()
}

#[cfg(target_os = "windows")]
fn get_num_physical_cpus() -> usize {
    match get_num_physical_cpus_windows() {
        Some(num) => num,
        None => get_num_cpus(),
    }
}

#[cfg(target_os = "windows")]
fn get_num_physical_cpus_windows() -> Option<usize> {
    // Inspired by https://msdn.microsoft.com/en-us/library/ms683194

    use std::mem;
    use std::ptr;

    #[allow(non_upper_case_globals)]
    const RelationProcessorCore: u32 = 0;

    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
        mask: usize,
        relationship: u32,
        _unused: [u64; 2],
    }

    extern "system" {
        fn GetLogicalProcessorInformation(
            info: *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION,
            length: &mut u32,
        ) -> u32;
    }

    // First we need to determine how much space to reserve.

    // The required size of the buffer, in bytes.
    let mut needed_size = 0;

    unsafe {
        GetLogicalProcessorInformation(ptr::null_mut(), &mut needed_size);
    }

    let struct_size = mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION>() as u32;

    // Could be 0, or some other bogus size.
    if needed_size == 0 || needed_size < struct_size || needed_size % struct_size != 0 {
        return None;
    }

    let count = needed_size / struct_size;

    // Allocate some memory where we will store the processor info.
    let mut buf = Vec::with_capacity(count as usize);

    let result;

    unsafe {
        result = GetLogicalProcessorInformation(buf.as_mut_ptr(), &mut needed_size);
    }

    // Failed for any reason.
    if result == 0 {
        return None;
    }

    let count = needed_size / struct_size;

    unsafe {
        buf.set_len(count as usize);
    }

    let phys_proc_count = buf
        .iter()
        // Only interested in processor packages (physical processors.)
        .filter(|proc_info| proc_info.relationship == RelationProcessorCore)
        .count();

    if phys_proc_count == 0 {
        None
    } else {
        Some(phys_proc_count)
    }
}

#[cfg(windows)]
fn get_num_cpus() -> usize {
    #[repr(C)]
    struct SYSTEM_INFO {
        wProcessorArchitecture: u16,
        wReserved: u16,
        dwPageSize: u32,
        lpMinimumApplicationAddress: *mut u8,
        lpMaximumApplicationAddress: *mut u8,
        dwActiveProcessorMask: *mut u8,
        dwNumberOfProcessors: u32,
        dwProcessorType: u32,
        dwAllocationGranularity: u32,
        wProcessorLevel: u16,
        wProcessorRevision: u16,
    }

    extern "system" {
        fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
    }

    unsafe {
        let mut sysinfo: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut sysinfo);
        sysinfo.dwNumberOfProcessors as usize
    }
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd"))]
fn get_num_cpus() -> usize {
    use std::ptr;

    let mut cpus: libc::c_uint = 0;
    let mut cpus_size = std::mem::size_of_val(&cpus);

    unsafe {
        cpus = libc::sysconf(libc::_SC_NPROCESSORS_ONLN) as libc::c_uint;
    }
    if cpus < 1 {
        let mut mib = [libc::CTL_HW, libc::HW_NCPU, 0, 0];
        unsafe {
            libc::sysctl(
                mib.as_mut_ptr(),
                2,
                &mut cpus as *mut _ as *mut _,
                &mut cpus_size as *mut _ as *mut _,
                ptr::null_mut(),
                0,
            );
        }
        if cpus < 1 {
            cpus = 1;
        }
    }
    cpus as usize
}

#[cfg(target_os = "openbsd")]
fn get_num_cpus() -> usize {
    use std::ptr;

    let mut cpus: libc::c_uint = 0;
    let mut cpus_size = std::mem::size_of_val(&cpus);
    let mut mib = [libc::CTL_HW, libc::HW_NCPUONLINE, 0, 0];
    let rc: libc::c_int;

    unsafe {
        rc = libc::sysctl(
            mib.as_mut_ptr(),
            2,
            &mut cpus as *mut _ as *mut _,
            &mut cpus_size as *mut _ as *mut _,
            ptr::null_mut(),
            0,
        );
    }
    if rc < 0 {
        cpus = 1;
    }
    cpus as usize
}

#[cfg(target_os = "openbsd")]
fn get_num_physical_cpus() -> usize {
    use std::ptr;

    let mut cpus: libc::c_uint = 0;
    let mut cpus_size = std::mem::size_of_val(&cpus);
    let mut mib = [libc::CTL_HW, libc::HW_NCPU, 0, 0];
    let rc: libc::c_int;

    unsafe {
        rc = libc::sysctl(
            mib.as_mut_ptr(),
            2,
            &mut cpus as *mut _ as *mut _,
            &mut cpus_size as *mut _ as *mut _,
            ptr::null_mut(),
            0,
        );
    }
    if rc < 0 {
        cpus = 1;
    }
    cpus as usize
}

#[cfg(target_os = "macos")]
fn get_num_physical_cpus() -> usize {
    use std::ffi::CStr;
    use std::ptr;

    let mut cpus: i32 = 0;
    let mut cpus_size = std::mem::size_of_val(&cpus);

    let sysctl_name =
        CStr::from_bytes_with_nul(b"hw.physicalcpu\0").expect("byte literal is missing NUL");

    unsafe {
        if 0 != libc::sysctlbyname(
            sysctl_name.as_ptr(),
            &mut cpus as *mut _ as *mut _,
            &mut cpus_size as *mut _ as *mut _,
            ptr::null_mut(),
            0,
        ) {
            return get_num_cpus();
        }
    }
    cpus as usize
}

#[cfg(target_os = "aix")]
fn get_num_physical_cpus() -> usize {
    match get_smt_threads_aix() {
        Some(num) => get_num_cpus() / num,
        None => get_num_cpus(),
    }
}

#[cfg(target_os = "aix")]
fn get_smt_threads_aix() -> Option<usize> {
    let smt = unsafe { libc::getsystemcfg(libc::SC_SMT_TC) };
    if smt == u64::MAX {
        return None;
    }
    Some(smt as usize)
}

#[cfg(any(
    target_os = "nacl",
    target_os = "macos",
    target_os = "ios",
    target_os = "android",
    target_os = "aix",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "fuchsia"
))]
fn get_num_cpus() -> usize {
    // On ARM targets, processors could be turned off to save power.
    // Use `_SC_NPROCESSORS_CONF` to get the real number.
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    const CONF_NAME: libc::c_int = libc::_SC_NPROCESSORS_CONF;
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    const CONF_NAME: libc::c_int = libc::_SC_NPROCESSORS_ONLN;

    let cpus = unsafe { libc::sysconf(CONF_NAME) };
    if cpus < 1 {
        1
    } else {
        cpus as usize
    }
}

#[cfg(target_os = "haiku")]
fn get_num_cpus() -> usize {
    use std::mem;

    #[allow(non_camel_case_types)]
    type bigtime_t = i64;
    #[allow(non_camel_case_types)]
    type status_t = i32;

    #[repr(C)]
    pub struct system_info {
        pub boot_time: bigtime_t,
        pub cpu_count: u32,
        pub max_pages: u64,
        pub used_pages: u64,
        pub cached_pages: u64,
        pub block_cache_pages: u64,
        pub ignored_pages: u64,
        pub needed_memory: u64,
        pub free_memory: u64,
        pub max_swap_pages: u64,
        pub free_swap_pages: u64,
        pub page_faults: u32,
        pub max_sems: u32,
        pub used_sems: u32,
        pub max_ports: u32,
        pub used_ports: u32,
        pub max_threads: u32,
        pub used_threads: u32,
        pub max_teams: u32,
        pub used_teams: u32,
        pub kernel_name: [::std::os::raw::c_char; 256usize],
        pub kernel_build_date: [::std::os::raw::c_char; 32usize],
        pub kernel_build_time: [::std::os::raw::c_char; 32usize],
        pub kernel_version: i64,
        pub abi: u32,
    }

    extern "C" {
        fn get_system_info(info: *mut system_info) -> status_t;
    }

    let mut info: system_info = unsafe { mem::zeroed() };
    let status = unsafe { get_system_info(&mut info as *mut _) };
    if status == 0 {
        info.cpu_count as usize
    } else {
        1
    }
}

#[cfg(target_os = "hermit")]
fn get_num_cpus() -> usize {
    unsafe { hermit_abi::get_processor_count() }
}

#[cfg(not(any(
    target_os = "nacl",
    target_os = "macos",
    target_os = "ios",
    target_os = "android",
    target_os = "aix",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "fuchsia",
    target_os = "linux",
    target_os = "openbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "haiku",
    target_os = "hermit",
    feature = "axstd",
    windows,
)))]
fn get_num_cpus() -> usize {
    1
}