mod lib;

fn main() {
    let args = lib::args::Args::parse();

    println!("{:#?}", args);
}
