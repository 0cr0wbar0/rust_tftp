use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::*;
use std::path::Path;
use bytes::Bytes;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();
    let (packet, client_addr) = Packet::receive(&socket);
    socket.connect(client_addr).unwrap();
    println!("Connection received from {}", client_addr);
    match packet {
        Packet::RrqPacket {
            opcode: Opcode::RRQ, filename, ..
        } => {
            let read_file_path = Path::new(&filename);
            if read_file_path.exists() {
                let data = Bytes::from(fs::read(filename).unwrap());
                Packet::send_file(data, &socket).expect("Wrong packet type received!");
            } else {
                let err = Packet::ErrPacket {
                    opcode: Opcode::ERR,
                    err_code: 1,
                    err_msg: "File not found".to_string()
                };
                err.send(&socket);
            }
        }
        Packet::WrqPacket {
            opcode: Opcode::WRQ, filename, ..
        } => {
            let write_file_path = Path::new(&filename);
            if write_file_path.exists() {
                let err = Packet::ErrPacket {
                    opcode: Opcode::ERR,
                    err_code: 6,
                    err_msg: "File already exists".to_string()
                };
                err.send(&socket);
            } else {
                let new_packet = Packet::AckPacket {
                    opcode: Opcode::ACK,
                    block_no: 0
                };
                new_packet.send(&socket);
                let file_bytes = Packet::receive_file(&socket);
                let mut file = File::create(filename).unwrap();
                file.write_all(&file_bytes).unwrap();
            }
        }
        _ => {
            panic!("Opcode error")
        }
    }
}