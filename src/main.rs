use rpeg::decoder::Decoder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{:#?}", args);
    if args.len() > 2 {
        panic!("[E] - too many arguments");
    }
    if args.len() < 2 {
        panic!("[E] - not enough arguments");
    }
    let mut d = Decoder::new(&args[1]);
    d.debug();
}
