use nix::fcntl;
use nix::unistd;
use nix::sys::stat;
use nix::fcntl::OFlag;

pub struct Strpipe {
    path: String,
    fd: i32,
    recv_buf: [u8; 512],
    main_buf: Vec<u8>,
}

impl Strpipe {
    pub fn new(path: &str) -> Self {

        // if pipe doesn't exist then create it
        if stat::stat(path).is_err() {
            unistd::mkfifo(path, stat::Mode::S_IRWXU).unwrap();
        }

        // open pipe
        let fd = fcntl::open(path, OFlag::O_RDWR, stat::Mode::S_IRWXU).unwrap();

        Strpipe {
            path: path.to_owned(),
            fd,
            recv_buf: [0u8; 512],
            main_buf: vec![],
        }
    }

    pub fn read<F: Fn(&str)>(&mut self, callback: F) {
        // accepts fn that accepts &str
        // executes that fn when data arrives
        let len = unistd::read(self.fd, &mut self.recv_buf).unwrap();
        self.main_buf.extend(&self.recv_buf[..len]);
        while let Some(idx) = self.main_buf.iter().position(|&i| i == b'\r' || i == b'\n') {
            if idx == 0 {
                self.main_buf.drain(..1);
                continue;
            }
            if let Ok(line) = std::str::from_utf8(&self.main_buf[..idx]) {
                let line = line.trim();
                if line.is_empty() { break; }
                callback(line);
                self.main_buf.drain(..=idx);
            }
        }
    }
}

impl Drop for Strpipe {
    fn drop(&mut self) {
        unistd::close(self.fd).unwrap();
        unistd::unlink(self.path.as_str()).unwrap();
    }
}
