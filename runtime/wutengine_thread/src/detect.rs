//! Thread configuration detection

use core::num::NonZero;

use smallvec::SmallVec;

/// The CPU core configuration of the current machine
#[derive(Debug, Clone)]
pub(super) struct CoreConfig {
    /// Number of physical cores
    #[expect(unused, reason = "Might be useful later")]
    pub(super) cores: NonZero<usize>,

    /// Number of logical cores
    pub(super) threads: NonZero<usize>,

    /// Number of threads per performance class. Higher indices have
    /// more performance
    pub(super) threads_by_class: SmallVec<[usize; 2]>,
}

/// Tries to find the CPU configuration of the machine we're running on
pub(super) fn try_detect_core_config() -> Option<CoreConfig> {
    profiling::function_scope!();

    cfg_select! {
        windows => {
            win::try_detect_core_config()
        }
        target_os = "macos" => {
            macos::try_detect_core_config()
        }
        _ => {
            log::debug!("Core count detection not available on current platform");
            None
        }
    }
}

#[cfg(windows)]
mod win {
    use core::num::NonZero;

    use nohash_hasher::IntSet;
    use smallvec::SmallVec;
    use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
    use windows::Win32::System::SystemInformation::CpuSetInformation;
    use windows::Win32::System::SystemInformation::GetSystemCpuSetInformation;
    use windows::Win32::System::SystemInformation::SYSTEM_CPU_SET_INFORMATION;
    use windows::core::HRESULT;

    use super::CoreConfig;

    pub(super) fn try_detect_core_config() -> Option<CoreConfig> {
        let mut num_bytes = 0;

        let result =
            unsafe { GetSystemCpuSetInformation(None, 0, &raw mut num_bytes, None, None) }.ok();

        if let Err(e) = result
            && e.code() != HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0)
        {
            log::warn!("Failed to get CPU set info size: {e}");
            return None;
        }

        assert_ne!(0, num_bytes, "Cannot allocate 0 bytes");

        let mut buffer = vec![0u8; num_bytes as usize];

        let buf_ptr = buffer.as_mut_ptr();
        let cpu_set_ptr = buf_ptr as *mut SYSTEM_CPU_SET_INFORMATION;

        let mut valid_bytes = 0;
        let result = unsafe {
            GetSystemCpuSetInformation(
                Some(cpu_set_ptr),
                num_bytes,
                &raw mut valid_bytes,
                None,
                None,
            )
        }
        .ok();

        let buffer = buffer; // mut -> readonly

        if let Err(e) = result {
            log::warn!("Failed to get CPU set info: {e}");
            return None;
        };

        let mut cur_byte_offset = 0;

        let mut cores = IntSet::default();
        let mut threads = IntSet::default();
        let mut by_class = SmallVec::new_const();

        while cur_byte_offset < buffer.len() {
            let info_ptr = unsafe { buffer.as_ptr().byte_add(cur_byte_offset) }
                as *const SYSTEM_CPU_SET_INFORMATION;
            let info = unsafe { info_ptr.as_ref() }.unwrap();

            assert!(
                cur_byte_offset + (info.Size as usize) <= buffer.len(),
                "Buffer overflow"
            );

            cur_byte_offset += info.Size as usize;

            if info.Type != CpuSetInformation {
                continue;
            }

            let cpu_set = unsafe { info.Anonymous.CpuSet };

            cores.insert(combine_group_index(cpu_set.Group, cpu_set.CoreIndex));
            threads.insert(combine_group_index(
                cpu_set.Group,
                cpu_set.LogicalProcessorIndex,
            ));

            while by_class.len() < (cpu_set.EfficiencyClass as usize + 1) {
                by_class.push(0);
            }

            by_class[cpu_set.EfficiencyClass as usize] += 1;
        }

        let (Some(cores), Some(threads)) = (NonZero::new(cores.len()), NonZero::new(threads.len()))
        else {
            return None;
        };

        Some(CoreConfig {
            cores,
            threads,
            threads_by_class: by_class,
        })
    }

    const fn combine_group_index(group: u16, index: u8) -> u32 {
        ((group as u32) << 16) | (index as u32)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use alloc::ffi::CString;
    use core::ffi::CStr;
    use core::ffi::c_char;
    use core::num::NonZero;
    use core::ptr::null_mut;
    use std::os::raw::c_void;

    use smallvec::SmallVec;

    use super::CoreConfig;

    fn errno_string(errno: libc::mach_error_t) -> Option<String> {
        let errno_cstr: *const c_char = unsafe { libc::strerror(errno) };

        unsafe { CStr::from_ptr(errno_cstr) }
            .to_str()
            .ok()
            .map(|s| s.to_string())
    }

    fn sysctl_u32(name: &str) -> Option<u32> {
        let name_c = CString::new(name).expect("Invalid sysctl name");

        let mut len: libc::size_t = 0;
        let errno =
            unsafe { libc::sysctlbyname(name_c.as_ptr(), null_mut(), &raw mut len, null_mut(), 0) };

        if errno != 0 {
            log::warn!(
                "Could not read sysctl length to determine core count: {}",
                errno_string(errno).unwrap_or_else(|| "<UNKNOWN_ERROR>".to_string())
            );
            return None;
        }

        assert_eq!(size_of::<u32>(), len as usize, "Unexpected sysctl size");

        let mut value: u32 = 0;
        let mut len_u32: libc::size_t = size_of::<u32>();

        let errno = unsafe {
            libc::sysctlbyname(
                name_c.as_ptr(),
                &raw mut value as *mut c_void,
                &raw mut len_u32,
                null_mut(),
                0,
            )
        };

        if errno != 0 {
            log::warn!(
                "Could not read sysctl length to determine core count: {}",
                errno_string(errno).unwrap_or_else(|| "<UNKNOWN_ERROR>".to_string())
            );
            return None;
        }

        assert_eq!(
            size_of::<u32>(),
            len_u32 as usize,
            "Unexpected sysctl size after read"
        );

        Some(value)
    }

    pub(super) fn try_detect_core_config() -> Option<CoreConfig> {
        let cores = NonZero::new(sysctl_u32("hw.physicalcpu")? as usize)?;
        let threads = NonZero::new(sysctl_u32("hw.logicalcpu")? as usize)?;
        let num_levels = sysctl_u32("hw.nperflevels")?;

        let mut by_class = SmallVec::new_const();

        by_class.resize(num_levels as usize, 0);

        for level in 0..num_levels {
            let at_level = sysctl_u32(&format!("hw.perflevel{}.logicalcpu", level))?;
            by_class[level as usize] = at_level as usize;
        }

        // MacOS reports performance levels with smaller = better
        by_class.reverse();

        Some(CoreConfig {
            cores,
            threads,
            threads_by_class: by_class,
        })
    }
}
