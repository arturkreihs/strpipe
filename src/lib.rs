use nix::fcntl::OFlag;
use nix::sys::stat;
use nix::{fcntl, unistd};
use thiserror::Error;

pub struct Strpipe {
    path: String,
    fd: i32,
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

impl Strpipe {
    pub fn new(path: &str) -> Result<Self, StrpipeError> {
        // if pipe doesn't exist then create it
        if stat::stat(path).is_err() {
            unistd::mkfifo(path, stat::Mode::S_IRWXU)?;
        }

        // open pipe
        let fd = fcntl::open(path, OFlag::O_RDWR, stat::Mode::S_IRWXU)?;

        Ok(Strpipe {
            path: path.to_owned(),
            fd,
            recv_buf: [0u8; 512],
            main_buf: vec![],
        })
    }

    pub fn read<F: Fn(&str)>(&mut self, callback: F) -> Result<(), StrpipeError> {
        // accepts fn that accepts &str
        // executes that fn when data arrives
        let len = unistd::read(self.fd, &mut self.recv_buf)?;
        self.main_buf.extend(&self.recv_buf[..len]);
        while let Some(idx) = self.main_buf.iter().position(|&i| i == b'\r' || i == b'\n') {
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

impl Drop for Strpipe {
    fn drop(&mut self) {
        match (unistd::close(self.fd), unistd::unlink(self.path.as_str())) {
            (Err(_), _) => log::error!("closing"),
            (_, Err(_)) => log::error!("unlinking"),
            _ => (),
        }
    }
}
