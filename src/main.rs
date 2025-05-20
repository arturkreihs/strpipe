use std::path::Path;
use strpipe::Strpipe;

#[tokio::main]
async fn main() {
    let mut strpipe = Strpipe::new(Path::new("test")).unwrap();

    loop {
        println!("reading");
        strpipe.read(|line| Box::new(Box::pin(async move {println!("{line}");}))).await.unwrap();
    }
}
