use rpeg::decoder::Decoder;

fn main() {
    let mut d = Decoder::new(String::from("jpg-icon.jpeg"));
    d.debug();
}
