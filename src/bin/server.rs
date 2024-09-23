use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use std::path::Path;
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
            let server_dir = "server/".to_owned() + &filename;
            let read_file_path = Path::new(&server_dir);
            if read_file_path.exists() {
                let data = fs::read(server_dir).unwrap();
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
            let server_dir = "server/".to_owned() + &filename;
            let write_file_path = Path::new(&server_dir);
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
                let mut file = File::create(server_dir).unwrap();
                file.write_all(&file_bytes).unwrap();
            }
        }
        _ => {
            panic!("Opcode error")
        }
    }
}