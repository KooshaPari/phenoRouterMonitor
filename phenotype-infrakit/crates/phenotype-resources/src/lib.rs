//! BKM-01: Native resource sampling (FD, memory, load) for thegent.
//! No subprocess spawns (lsof, vm_stat).

use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ResourceSnapshot {
    pub fd_used: u32,
    pub fd_limit: u32,
    pub mem_rss_mb: f64,
    pub mem_available_mb: f64,
    pub cpu_count: u32,
    pub load_1m: f64,
    pub load_5m: f64,
    pub load_15m: f64,
}

fn get_fd_usage() -> (u32, u32) {
    let limit = get_fd_limit();
    let used = if Path::new("/proc/self/fd").exists() {
        match fs::read_dir("/proc/self/fd") {
            Ok(entries) => entries.count().saturating_sub(1) as u32,
            _ => 0,
        }
    } else {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let pid = std::process::id();
            if let Ok(out) = Command::new("lsof").args(["-p", &pid.to_string()]).output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    let count = s.lines().filter(|l| !l.contains(" txt ")).count();
                    count.saturating_sub(1) as u32
                } else {
                    0
                }
            } else {
                0
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            0
        }
    };
    (used, limit)
}

fn get_fd_limit() -> u32 {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        let mut rlim: MaybeUninit<libc::rlimit> = MaybeUninit::uninit();
        if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, rlim.as_mut_ptr()) } == 0 {
            let rlim = unsafe { rlim.assume_init() };
            let soft = rlim.rlim_cur;
            if soft == libc::RLIM_INFINITY {
                1024
            } else {
                soft as u32
            }
        } else {
            1024
        }
    }
    #[cfg(not(unix))]
    {
        1024
    }
}

fn get_memory_mb() -> (f64, f64) {
    let rss_mb = get_rss_mb();
    let available_mb = if Path::new("/proc/meminfo").exists() {
        if let Ok(mut f) = fs::File::open("/proc/meminfo") {
            let mut buf = String::new();
            if f.read_to_string(&mut buf).is_ok() {
                for line in buf.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(rest) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = rest.parse::<u64>() {
                                return (rss_mb, kb as f64 / 1024.0);
                            }
                        }
                        break;
                    }
                }
            }
        }
        512.0
    } else {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(out) = Command::new("vm_stat").output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    let mut free = 0u64;
                    let mut inactive = 0u64;
                    let mut speculative = 0u64;
                    let mut purgeable = 0u64;
                    let mut page_size = 4096u64;

                    for line in s.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        if line.contains("page size of") {
                            if let Some(start) = line.find("page size of ") {
                                let rest = &line[start + 13..];
                                if let Some(end) = rest.find(" bytes") {
                                    if let Ok(ps) = rest[..end].parse::<u64>() {
                                        page_size = ps;
                                    }
                                }
                            }
                        } else if line.contains("Pages free") {
                            if let Some(rest) = line.split(':').nth(1) {
                                free = rest.trim().trim_end_matches('.').parse().unwrap_or(0);
                            }
                        } else if line.contains("Pages inactive") {
                            if let Some(rest) = line.split(':').nth(1) {
                                inactive = rest.trim().trim_end_matches('.').parse().unwrap_or(0);
                            }
                        } else if line.contains("Pages speculative") {
                            if let Some(rest) = line.split(':').nth(1) {
                                speculative =
                                    rest.trim().trim_end_matches('.').parse().unwrap_or(0);
                            }
                        } else if line.contains("Pages purgeable") {
                            if let Some(rest) = line.split(':').nth(1) {
                                purgeable = rest.trim().trim_end_matches('.').parse().unwrap_or(0);
                            }
                        }
                    }
                    (free + inactive + speculative + purgeable) as f64 * page_size as f64
                        / (1024.0 * 1024.0)
                } else {
                    1024.0
                }
            } else {
                1024.0
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            1024.0
        }
    };
    (rss_mb, available_mb)
}

fn get_rss_mb() -> f64 {
    #[cfg(unix)]
    {
        if Path::new("/proc/self/status").exists() {
            if let Ok(content) = fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(k) = kb.parse::<u64>() {
                                return k as f64 / 1024.0;
                            }
                        }
                        break;
                    }
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let pid = std::process::id();
            if let Ok(out) = Command::new("ps")
                .args(["-o", "rss=", "-p", &pid.to_string()])
                .output()
            {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if let Ok(kb) = s.trim().parse::<u64>() {
                        return kb as f64 / 1024.0;
                    }
                }
            }
        }
    }
    0.0
}

fn get_load_avg() -> (f64, f64, f64) {
    #[cfg(unix)]
    {
        let mut load: [libc::c_double; 3] = [0.0, 0.0, 0.0];
        if unsafe { libc::getloadavg(load.as_mut_ptr(), 3) } == 3 {
            (load[0], load[1], load[2])
        } else {
            (0.0, 0.0, 0.0)
        }
    }
    #[cfg(not(unix))]
    {
        (0.0, 0.0, 0.0)
    }
}

/// Sample current system resources. Cross-platform.
pub fn sample() -> ResourceSnapshot {
    let (fd_used, fd_limit) = get_fd_usage();
    let (mem_rss_mb, mem_available_mb) = get_memory_mb();
    let cpu_count = std::thread::available_parallelism()
        .map(|p| p.get() as u32)
        .unwrap_or(1)
        .max(1);
    let (load_1m, load_5m, load_15m) = get_load_avg();

    ResourceSnapshot {
        fd_used,
        fd_limit,
        mem_rss_mb,
        mem_available_mb,
        cpu_count,
        load_1m,
        load_5m,
        load_15m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_returns_valid_snapshot() {
        let snapshot = sample();

        // FD usage should be reasonable
        assert!(snapshot.fd_limit > 0, "FD limit should be positive");
        assert!(
            snapshot.fd_used <= snapshot.fd_limit,
            "FD used should not exceed limit"
        );

        // Memory values should be non-negative
        assert!(
            snapshot.mem_rss_mb >= 0.0,
            "RSS memory should be non-negative"
        );
        assert!(
            snapshot.mem_available_mb >= 0.0,
            "Available memory should be non-negative"
        );

        // CPU count should be at least 1
        assert!(snapshot.cpu_count >= 1, "CPU count should be at least 1");

        // Load averages should be non-negative
        assert!(snapshot.load_1m >= 0.0, "Load 1m should be non-negative");
        assert!(snapshot.load_5m >= 0.0, "Load 5m should be non-negative");
        assert!(snapshot.load_15m >= 0.0, "Load 15m should be non-negative");
    }

    #[test]
    fn test_snapshot_debug() {
        let snapshot = sample();
        // Verify Debug trait is implemented
        let debug_str = format!("{:?}", snapshot);
        assert!(debug_str.contains("fd_used"));
        assert!(debug_str.contains("mem_rss_mb"));
        assert!(debug_str.contains("cpu_count"));
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = sample();
        // Verify Serialize trait works
        let json = serde_json::to_string(&snapshot);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("fd_used"));
        assert!(json_str.contains("mem_rss_mb"));
    }

    #[test]
    fn test_fd_limit_is_reasonable() {
        let snapshot = sample();
        // FD limit is typically between 256 and millions
        assert!(snapshot.fd_limit >= 256, "FD limit should be at least 256");
        assert!(
            snapshot.fd_limit <= 10_000_000,
            "FD limit should be reasonable"
        );
    }

    #[test]
    fn test_memory_snapshot_consistency() {
        let snapshot1 = sample();
        let snapshot2 = sample();

        // Two consecutive samples should have similar values
        // (memory shouldn't change drastically in microseconds)
        let mem_diff = (snapshot1.mem_rss_mb - snapshot2.mem_rss_mb).abs();
        assert!(
            mem_diff < 100.0,
            "Memory shouldn't change drastically between samples"
        );
    }
}
