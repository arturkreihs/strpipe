use strpipe::Strpipe;

fn main() {
    let mut strpipe = Strpipe::new("named_pipe");

    loop {
        println!("reading");
        strpipe.read(|line| println!("{line}"));
    }
}
