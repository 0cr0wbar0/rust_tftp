use std::io::Read;
use std::net::UdpSocket;
use bytes::{BufMut, BytesMut};

pub enum Opcode {
    RRQ = 1,
    WRQ = 2,
    DATA = 3,
    ACK = 4,
    ERR = 5
}

pub enum Mode {
    Octet
}

pub enum Packet {
    WrqPacket {
        opcode: Opcode,
        filename: String,
        mode: Mode
    },
    RrqPacket {
        opcode: Opcode,
        filename: String,
        mode: Mode
    },
    DataPacket {
        opcode: Opcode,
        block_no: u16,
        data: bytes::Bytes
    },
    AckPacket {
        opcode: Opcode,
        block_no: u16
    },
    ErrPacket {
        opcode: Opcode,
        err_code: u16,
        err_msg: String
    }
}

impl Packet {
    pub fn send(self, socket: &UdpSocket) {
        let mut buf = BytesMut::with_capacity(512);
        match self {
            Packet::WrqPacket {
                opcode,
                filename,
                ..
            } => {
                buf.put_u8(opcode as u8);
                buf.put(filename.as_bytes());
                buf.put_u8(0);
                buf.put(&b"octet"[..]);
                buf.put_u8(0);

            }
            Packet::RrqPacket {
                opcode,
                filename,
                ..
            } => {
                buf.put_u8(opcode as u8);
                buf.put(filename.as_bytes());
                buf.put_u8(0);
                buf.put(&b"octet"[..]);
                buf.put_u8(0);
            }
            Packet::DataPacket {
                opcode,
                block_no,
                data,
            } => {
                buf.put_u8(opcode as u8);
                buf.put_u16(block_no);
                buf.put(data);
            }
            Packet::AckPacket {
                opcode,
                block_no
            } => {
                buf.put_u8(opcode as u8);
                buf.put_u16(block_no);
            }
            Packet::ErrPacket {
                opcode,
                err_code,
                err_msg
            } => {
                buf.put_u8(opcode as u8);
                buf.put_u16(err_code);
                buf.put(err_msg.as_bytes());
                buf.put_u8(0);
            }
        }
        socket.send(&buf).unwrap();
    }
    pub fn receive(socket: &UdpSocket) -> Self {
        let mut buf = [0u8; 512];
        let result = socket.recv_from(&mut buf).unwrap();
        todo!()
    }
}