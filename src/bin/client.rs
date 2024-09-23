use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:9000").unwrap();
    socket.connect("0.0.0.0:8000").expect("Couldn't connect to server");
    let mut num_request = String::new();
    let mut file_request = String::new();
    println!("Select option: \n 1. send \n 2. receive");
    io::stdin().read_line(&mut num_request).unwrap();
    println!("Enter name of file: ");
    io::stdin().read_line(&mut file_request).unwrap();
    match num_request.trim().parse::<i32>().unwrap() {
        1 => {
            let write_packet = Packet::WrqPacket {
                opcode: Opcode::WRQ,
                filename: file_request.trim_end().to_string(),
                mode: Mode::Octet,
            };
            write_packet.send(&socket);
            let (recv_packet, server_addr) = Packet::receive(&socket);
            println!("Connection received from {}", server_addr);
            // need to match both packets at the same time in if-let
            // in order to access attributes from both at the same time
            // this negates the need to nest if-let statements
            if let (
                Packet::AckPacket {opcode: Opcode::ACK, ..},
                Packet::WrqPacket {filename, ..}
            ) = (recv_packet, &write_packet) {
                let client_dir = "client/".to_owned() + filename;
                let data = fs::read(client_dir).unwrap();
                Packet::send_file(data, &socket).expect("Wrong packet type received!");
            } else {
                dbg!(&write_packet);
            }
        }
        2 => {
            let read_packet = Packet::RrqPacket {
                opcode: Opcode::RRQ,
                filename: file_request.trim_end().to_string(),
                mode: Mode::Octet
            };
            read_packet.send(&socket);
            let file_bytes= Packet::receive_file(&socket);
            let client_dir = "client/".to_owned() + &file_request;
            let mut file = File::create(client_dir).unwrap();
            file.write_all(&file_bytes).unwrap();
        }
        _ => {
            panic!("Incorrect input, type 1 or 2 and then press enter")
        }
    }
}