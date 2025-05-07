use nix::fcntl::OFlag;
use nix::sys::stat;
use nix::{fcntl, unistd};
use thiserror::Error;
use std::os::fd::OwnedFd;
use std::path::Path;

#[derive(Debug)]
pub struct Strpipe<'a> {
    path: &'a Path,
    fd: OwnedFd,
    recv_buf: [u8; 512],
    main_buf: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum StrpipeError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("nix")]
    Nix(#[from] nix::errno::Errno),
}

impl<'a> Strpipe<'a> {
    pub fn new(path: &'a Path) -> Result<Self, StrpipeError> {
        // if pipe doesn't exist then create it
        if stat::stat(path).is_err() {
            unistd::mkfifo(path, stat::Mode::S_IRWXU)?;
        }

        // open pipe
        let fd = fcntl::open(path, OFlag::O_RDWR, stat::Mode::S_IRWXU)?;

        Ok(Strpipe {
            path,
            fd,
            recv_buf: [0u8; 512],
            main_buf: vec![],
        })
    }

    // accepts fn which accepts &str
    // calls that fn when data arrives
    pub fn read<F: FnMut(&str)>(&mut self, mut callback: F) -> Result<(), StrpipeError> {
        let len = unistd::read(&self.fd, &mut self.recv_buf)?;
        self.main_buf.extend(&self.recv_buf[..len]);
        while let Some(idx) = self.main_buf.iter()
            .position(|&i| i == b'\r' || i == b'\n') {
            if idx == 0 {
                self.main_buf.drain(..1);
                continue;
            }
            if let Ok(line) = std::str::from_utf8(&self.main_buf[..idx]) {
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                callback(line);
                self.main_buf.drain(..=idx);
            }
        }
        Ok(())
    }
}

impl Drop for Strpipe<'_> {
    fn drop(&mut self) {
        let fd = self.fd.try_clone().unwrap();
        match (unistd::close(fd), unistd::unlink(self.path.as_os_str())) {
            (Err(_), _) => log::error!("closing"),
            (_, Err(_)) => log::error!("unlinking"),
            _ => (),
        }
    }
}
