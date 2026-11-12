//! Memory usage of this process, read after each search depth.
//!
//! - Linux: current and peak RSS from `/proc/self/status` (`VmRSS`, `VmHWM`).
//! - Other Unix (macOS): peak RSS from `getrusage`; current RSS isn't available there
//!   without the Mach APIs, so it's reported as unknown.

/// Resident memory in bytes; `None` where the platform doesn't provide it.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
    pub rss_bytes: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
}

impl MemoryUsage {
    /// Current RSS if known, otherwise the peak (for display).
    pub fn best_rss(&self) -> Option<u64> {
        self.rss_bytes.or(self.peak_rss_bytes)
    }
}

pub fn current() -> MemoryUsage {
    let mut usage = MemoryUsage::default();
    #[cfg(target_os = "linux")]
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        usage.rss_bytes = status_kib(&status, "VmRSS:").map(|kib| kib * 1024);
        usage.peak_rss_bytes = status_kib(&status, "VmHWM:").map(|kib| kib * 1024);
    }
    if usage.peak_rss_bytes.is_none() {
        usage.peak_rss_bytes = getrusage_peak_bytes();
    }
    usage
}

/// A `kB` value from `/proc/self/status`.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn status_kib(status: &str, field: &str) -> Option<u64> {
    let line = status.lines().find(|l| l.starts_with(field))?;
    line[field.len()..].split_whitespace().next()?.parse().ok()
}

fn getrusage_peak_bytes() -> Option<u64> {
    // SAFETY: `getrusage` only writes into the zero-initialised struct we pass.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } != 0 {
        return None;
    }
    let max_rss = u64::try_from(usage.ru_maxrss).ok()?;
    // `ru_maxrss` is in bytes on macOS and in KiB on Linux and most other systems.
    Some(if cfg!(target_os = "macos") {
        max_rss
    } else {
        max_rss * 1024
    })
}

/// `1.23 GB`, `172 MB`, `950 kB` (powers of 1024).
pub fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    let b = bytes as f64;
    if b >= KIB * KIB * KIB {
        format!("{:.2} GB", b / (KIB * KIB * KIB))
    } else if b >= KIB * KIB {
        format!("{:.0} MB", b / (KIB * KIB))
    } else {
        format!("{:.0} kB", b / KIB)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_proc_status_fields() {
        let status = "Name:\tsegment\nVmHWM:\t  204800 kB\nVmRSS:\t  102400 kB\nThreads:\t17\n";
        assert_eq!(status_kib(status, "VmRSS:"), Some(102_400));
        assert_eq!(status_kib(status, "VmHWM:"), Some(204_800));
        assert_eq!(status_kib(status, "VmSwap:"), None);
    }

    #[test]
    fn reports_some_peak_memory() {
        let usage = current();
        let peak = usage.peak_rss_bytes.expect("peak RSS should be available on unix");
        assert!(peak > 1024 * 1024, "peak RSS {peak} implausibly small");
    }

    #[test]
    fn formats_bytes() {
        assert_eq!(format_bytes(950 * 1024), "950 kB");
        assert_eq!(format_bytes(172 * 1024 * 1024), "172 MB");
        assert_eq!(format_bytes(4_423_000_000), "4.12 GB");
    }
}
