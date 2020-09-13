#[path = "tcp.rs"]
mod tcp;

#[path = "udp.rs"]
mod udp;

#[path = "rtu.rs"]
mod rtu;

fn main() {
    tcp::tcpserver(1, &"127.0.0.1:5502");
    //udp::udpserver(1, &"127.0.0.1:5502");
    //rtu::rtuserver(1, &"/dev/ttyS0");
}

include!("tcp.rs");
