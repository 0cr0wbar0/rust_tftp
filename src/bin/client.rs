use std::net::*;
use std::io;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();
    let mut buf = [0u8; 512];
    let mut request = String::new();
    println!("{}", "Select option: \n 1. send \n 2. receive");
    io::stdin().read_line(&mut request).unwrap();
    match request.parse() {
        Ok(1) => {
            let mut packet = Packet::WrqPacket {
                opcode: Opcode::WRQ,
                filename: "test.txt".to_string(),
                mode: Mode::Octet,
            };
            packet.send(&socket);
        }
        Ok(2) => {

        }
        _ => {}
    }
}
