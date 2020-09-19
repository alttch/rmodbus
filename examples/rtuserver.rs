#[path = "servers/rtu.rs"]
mod rtu;

fn main() {
    rtu::rtuserver(1, &"/dev/ttyS0");
}
