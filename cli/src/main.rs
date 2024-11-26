fn run(bytes: impl Iterator<Item = u8>) -> std::io::Result<()> {
    todo!()
}
fn run_file(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    let mut buf = vec![];
    let mut file = std::fs::File::open(path)?;
    let _bytes = std::io::Read::read_to_end(&mut file, &mut buf)?;
    run(buf.into_iter())
}
fn main() {
    println!("Hello, world!");
}
