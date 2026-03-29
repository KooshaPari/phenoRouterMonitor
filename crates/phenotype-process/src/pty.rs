//! PTY (Pseudo-Terminal) management.
//!
//! Provides cross-platform PTY creation and management.

use std::fmt;

/// PTY size configuration.
#[derive(Debug, Clone, Copy)]
pub struct PtySize {
    /// Number of columns.
    pub cols: u16,
    /// Number of rows.
    pub rows: u16,
    /// Pixel width (optional).
    pub width: Option<u16>,
    /// Pixel height (optional).
    pub height: Option<u16>,
}

impl PtySize {
    /// Create a new PTY size.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            width: None,
            height: None,
        }
    }

    /// Create with pixel dimensions.
    pub fn with_pixels(mut self, width: u16, height: u16) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Default size (80x24).
    pub fn default_size() -> Self {
        Self::new(80, 24)
    }
}

impl Default for PtySize {
    fn default() -> Self {
        Self::default_size()
    }
}

/// PTY master side.
#[derive(Debug)]
pub struct PtyMaster {
    fd: i32,
    path: Option<String>,
}

impl PtyMaster {
    /// Create a new PTY master.
    #[cfg(unix)]
    pub fn new() -> std::io::Result<(Self, String)> {
        use std::fs::File;
        use std::os::fd::FromRawFd;
        use libc::{cfmakeraw, winsize, TIOCGWINSZ, TIOCSWINSZ, grantpt, unlockpt};

        // Open PTY master
        let fd = unsafe { libc::open("/dev/ptmx", libc::O_RDWR | libc::O_NOCTTY) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        // Grant access to slave
        if unsafe { grantpt(fd) } != 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error());
        }

        // Unlock PTY
        if unsafe { unlockpt(fd) } != 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error());
        }

        // Get PTY path
        let path = unsafe { libc::ptsname(fd) };
        if path.is_null() {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error());
        }

        let path_str = unsafe { std::ffi::CStr::from_ptr(path) }
            .to_string_lossy()
            .to_string();

        Ok((Self { fd, path: Some(path_str.clone()) }, path_str))
    }

    /// Resize the PTY.
    #[cfg(unix)]
    pub fn resize(&self, size: PtySize) -> std::io::Result<()> {
        use libc::{winsize, TIOCSWINSZ};

        let ws = winsize {
            ws_row: size.rows,
            ws_col: size.cols,
            ws_xpixel: size.width.unwrap_or(0),
            ws_ypixel: size.height.unwrap_or(0),
        };

        if unsafe { libc::ioctl(self.fd, TIOCSWINSZ, &ws) } < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    /// Write to the PTY.
    pub fn write(&self, data: &[u8]) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use libc::write;
            let n = unsafe { write(self.fd, data.as_ptr() as *const _, data.len()) };
            if n < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(n as usize)
            }
        }
        #[cfg(windows)]
        {
            unimplemented!("PTY write on Windows not yet implemented")
        }
    }

    /// Read from the PTY.
    pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use libc::read;
            let n = unsafe { read(self.fd, buf.as_mut_ptr() as *mut _, buf.len()) };
            if n < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(n as usize)
            }
        }
        #[cfg(windows)]
        {
            unimplemented!("PTY read on Windows not yet implemented")
        }
    }

    /// Close the PTY master.
    #[cfg(unix)]
    pub fn close(self) {
        unsafe { libc::close(self.fd) };
    }
}

/// PTY pair (master and slave).
pub struct PtyPair {
    master: PtyMaster,
    slave_path: String,
    slave_fd: i32,
}

impl PtyPair {
    /// Create a new PTY pair.
    #[cfg(unix)]
    pub fn new() -> std::io::Result<Self> {
        let (master, slave_path) = PtyMaster::new()?;
        let slave_fd = unsafe { libc::open(slave_path.as_ptr(), libc::O_RDWR) };
        if slave_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self {
            master,
            slave_path,
            slave_fd,
        })
    }

    /// Get the master side.
    pub fn master(&self) -> &PtyMaster {
        &self.master
    }

    /// Get the slave path.
    pub fn slave_path(&self) -> &str {
        &self.slave_path
    }

    /// Get the slave file descriptor.
    #[cfg(unix)]
    pub fn slave_fd(&self) -> i32 {
        self.slave_fd
    }

    /// Resize both sides of the PTY.
    pub fn resize(&self, size: PtySize) -> std::io::Result<()> {
        self.master.resize(size)
    }
}

impl Drop for PtyPair {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::close(self.slave_fd);
        }
    }
}

impl Drop for PtyMaster {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::close(self.fd);
        }
    }
}
