#[path = "servers/ascii.rs"]
mod ascii;

fn main() {
    ascii::asciiserver(1, &"/dev/ttyS0");
}
