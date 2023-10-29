use strpipe::Strpipe;

fn main() {
    let mut strpipe = Strpipe::new("orkpipe");

    loop {
        strpipe.read(|line| println!("{line}"));
    }
}
