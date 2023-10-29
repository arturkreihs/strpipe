use nix::fcntl;
use nix::unistd;
use nix::sys::stat;
use nix::fcntl::OFlag;

struct Strpipe {
    name: String,
    fd: i32,
    recv_buf: [u8; 512],
    main_buf: Vec<u8>,
}

impl Strpipe {
    pub fn new(name: &str) -> Self {

        // if pipe doesn't exist then create it
        if stat::stat(name).is_err() {
            unistd::mkfifo(name, stat::Mode::S_IRWXU).unwrap();
        }

        // open pipe
        let fd = fcntl::open(name, OFlag::O_RDWR, stat::Mode::S_IRWXU).unwrap();

        Strpipe {
            name: name.to_owned(),
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
        while let Some(idx) = self.main_buf.iter().position(|&i| i == 0x0d) {
            if let Ok(line) = std::str::from_utf8(&self.main_buf[..idx]) {
                let line = line.trim(); //needed for 0x0a removal
                if line.is_empty() { break; }
//                println!("\"{line}\"");
                callback(line);
                self.main_buf.drain(..=idx);
            }
        }
    }
}

impl Drop for Strpipe {
    fn drop(&mut self) {
        unistd::close(self.fd).unwrap();
        unistd::unlink(self.name.as_str()).unwrap();
    }
}
