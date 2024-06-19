use std::io;
use std::net::*;
use tftp::Packet;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();
    let mut buf = [0u8; 512];

}