use std::{fs, io};
use std::net::*;
use bytes::Bytes;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:9000").unwrap();
    socket.connect("0.0.0.0:8000").expect("Couldn't connect to server");
    let mut num_request = String::new();
    let mut file_request = String::new();
    println!("{}", "Select option: \n 1. send \n 2. receive");
    io::stdin().read_line(&mut num_request).unwrap();
    println!("{}", "Enter name of file: ");
    io::stdin().read_line(&mut file_request).unwrap();
    match num_request.trim().parse::<i32>().unwrap() {
        1 => {
            let mut write_packet = Packet::WrqPacket {
                opcode: Opcode::WRQ,
                filename: file_request,
                mode: Mode::Octet,
            };
            write_packet.send(&socket);
            let (recv_packet, _) = Packet::receive(&socket);
            // need to match both packets at the same time in if-let
            // in order to access attributes from both at the same time
            // this negates the need to nest if-let statements
            if let (
                Packet::AckPacket {opcode: Opcode::ACK, ..},
                Packet::WrqPacket {filename, ..}
            ) = (recv_packet, &write_packet) {
                let data = Bytes::from(fs::read(filename).unwrap());
                let new_write_packet = Packet::DataPacket {
                    opcode: Opcode::DATA,
                    block_no: 1,
                    data
                };
                new_write_packet.send(&socket);
            } else {
                dbg!(&write_packet);
            }
        }
        2 => {
            let mut read_packet = Packet::RrqPacket {
                opcode: Opcode::RRQ,
                filename: "read_test.txt".to_string(),
                mode: Mode::Octet
            };
            read_packet.send(&socket);
            (read_packet, _) = Packet::receive(&socket);
            dbg!(&read_packet);
        }
        _ => {
            // todo!("Handle incorrect input")
        }
    }
}
