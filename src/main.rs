use rpeg::decoder::Decoder;

fn main() {
    let d = Decoder::new(String::from("simple.jpg"));
    d.debug();
}
