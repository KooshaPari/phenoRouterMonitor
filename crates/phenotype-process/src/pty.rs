//! Pseudo-terminal (PTY) utilities for interactive process handling.

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::path::PathBuf;

use super::error::{ProcessError, Result};

/// PTY master/slave pair.
#[derive(Debug)]
pub struct PtyPair {
    master: PtyMaster,
    slave: PtySlave,
}

impl PtyPair {
    /// Creates a new PTY pair.
    pub fn open() -> Result<Self> {
        #[cfg(unix)]
        {
            let master_fd = unsafe {
                let mut master_fd: RawFd = -1;
                let mut slave_fd: RawFd = -1;
                let mut win_size = libc::winsize {
                    ws_row: 24,
                    ws_col: 80,
                    ws_xpixel: 0,
                    ws_ypixel: 0,
                };

                let result = libc::openpty(
                    &mut master_fd,
                    &mut slave_fd,
                    std::ptr::null_mut(),
                    &mut win_size,
                    std::ptr::null(),
                );

                if result < 0 {
                    return Err(ProcessError::PtyOpen(
                        std::io::Error::last_os_error().to_string(),
                    ));
                }

                master_fd
            };

            Ok(Self {
                master: PtyMaster::from_raw_fd(master_fd),
                slave: PtySlave::from_raw_fd(slave_fd),
            })
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::NotSupported(
                "PTY is only supported on Unix systems".into(),
            ))
        }
    }

    /// Returns the master file descriptor.
    pub fn master(&self) -> &PtyMaster {
        &self.master
    }

    /// Returns the slave file descriptor.
    pub fn slave(&self) -> &PtySlave {
        &self.slave
    }

    /// Returns the path to the slave device.
    pub fn slave_path(&self) -> PathBuf {
        self.slave.path()
    }
}

/// PTY master endpoint.
#[derive(Debug)]
pub struct PtyMaster {
    fd: RawFd,
}

impl PtyMaster {
    /// Reads from the PTY master.
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        #[cfg(unix)]
        {
            let n = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n < 0 {
                Err(ProcessError::Io(std::io::Error::last_os_error()))
            } else {
                Ok(n as usize)
            }
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::NotSupported("PTY not supported".into()))
        }
    }

    /// Writes to the PTY master.
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        #[cfg(unix)]
        {
            let n =
                unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
            if n < 0 {
                Err(ProcessError::Io(std::io::Error::last_os_error()))
            } else {
                Ok(n as usize)
            }
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::NotSupported("PTY not supported".into()))
        }
    }

    /// Resizes the PTY window.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        #[cfg(unix)]
        {
            let win_size = libc::winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };

            if unsafe { libc::ioctl(self.fd, libc::TIOCSWINSZ, &win_size) } < 0 {
                Err(ProcessError::Io(std::io::Error::last_os_error()))
            } else {
                Ok(())
            }
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::NotSupported("PTY not supported".into()))
        }
    }
}

impl AsRawFd for PtyMaster {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for PtyMaster {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }
}

impl IntoRawFd for PtyMaster {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        std::mem::forget(self);
        fd
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

/// PTY slave endpoint.
#[derive(Debug)]
pub struct PtySlave {
    fd: RawFd,
    path: PathBuf,
}

impl PtySlave {
    /// Returns the path to this slave device.
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl AsRawFd for PtySlave {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for PtySlave {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        #[cfg(unix)]
        {
            let path = unsafe {
                let mut buffer: [libc::c_char; 128] = std::mem::zeroed();
                if libc::ttyname_r(fd, buffer.as_mut_ptr(), buffer.len()) == 0 {
                    PathBuf::from(
                        std::ffi::CStr::from_ptr(buffer.as_ptr())
                            .to_string_lossy()
                            .into_owned(),
                    )
                } else {
                    PathBuf::from(format!("/dev/pts/{}", fd))
                }
            };

            Self { fd, path }
        }

        #[cfg(not(unix))]
        {
            Self {
                fd,
                path: PathBuf::from("/dev/null"),
            }
        }
    }
}

impl IntoRawFd for PtySlave {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        std::mem::forget(self);
        fd
    }
}

impl Drop for PtySlave {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::close(self.fd);
        }
    }
}

/// Configures a file descriptor as non-blocking.
pub fn set_nonblocking(fd: RawFd) -> Result<()> {
    #[cfg(unix)]
    {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
        if flags < 0 {
            return Err(ProcessError::Io(std::io::Error::last_os_error()));
        }

        if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(ProcessError::Io(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        Err(ProcessError::NotSupported(
            "Non-blocking PTY not supported".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_pty_open() {
        let pair = PtyPair::open().expect("should open PTY");
        assert!(pair.slave_path().exists());
    }
}
