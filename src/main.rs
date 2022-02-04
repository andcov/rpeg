use rpeg::decoder::Decoder;

fn main() {
    let mut d = Decoder::new(String::from("bfly.jpg"));
    d.debug();
}
