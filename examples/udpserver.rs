#[path = "servers/udp.rs"]
mod udp;

fn main() {
    udp::udpserver(1, "127.0.0.1:5502");
}
