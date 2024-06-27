use std::io;
use std::net::*;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();
    let mut buf = [0u8; 512];
    let mut packet = Packet::receive(&socket);

}