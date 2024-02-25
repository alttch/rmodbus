#[path = "servers/tcp.rs"]
mod tcp;

fn main() {
    tcp::tcpserver(1, "127.0.0.1:5502");
}
