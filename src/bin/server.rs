use std::fs;
use std::net::*;
use std::path::Path;
use bytes::Bytes;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();
    let (packet, client_addr) = Packet::receive(&socket);
    socket.connect(client_addr).unwrap();
    match packet {
        Packet::RrqPacket {
            opcode: Opcode::RRQ, filename, ..
        } => {
            let read_file_path = Path::new(&filename);
            if read_file_path.exists() {
                let data = Bytes::from(fs::read(filename).unwrap());
                assert!(data.len() < 509, "Larger files are not handled currently");
                let new_packet = Packet::DataPacket {
                    opcode: Opcode::DATA,
                    block_no: 0,
                    data,
                };
                new_packet.send(&socket);
                // todo!("Packet sending loop")
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
                    err_code: 2,
                    err_msg: "File already exists".to_string()
                };
                err.send(&socket);
            } else {
                let new_packet = Packet::AckPacket {
                    opcode: Opcode::ACK,
                    block_no: 0
                };
                new_packet.send(&socket);
                let (write_packet, _) = Packet::receive(&socket);
                dbg!(&write_packet);
            }
        }
        _ => {
            panic!("Opcode error")
        }
    }
}