use std::path::Path;
use strpipe::Strpipe;

fn main() {
    let mut strpipe = Strpipe::new(Path::new("test")).unwrap();

    loop {
        println!("reading");
        strpipe.read(|line| println!("{line}")).unwrap();
    }
}
