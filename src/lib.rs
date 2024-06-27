use std::net::UdpSocket;
use bytes::{BufMut, Bytes, BytesMut};

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
        data: Bytes
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
        let mut received = BytesMut::with_capacity(512);
        socket.recv_from(&mut received).unwrap();
        let mut buf = Bytes::from(received);
        match buf[0] {
            1 => {
                return Packet::RrqPacket {
                    opcode: Opcode::RRQ,
                    filename: Self::extract_str(buf),
                    mode: Mode::Octet,
                };
            }
            2 => {
                return Packet::WrqPacket {
                    opcode: Opcode::RRQ,
                    filename: Self::extract_str(buf),
                    mode: Mode::Octet,
                };
            }
            3 => {
                return Packet::DataPacket {
                    opcode: Opcode::DATA,
                    // bitwise operation to convert two u8s into one u16, thanks stackoverflow :)
                    block_no: ((buf[1] as u16) << 8) | buf[2] as u16,
                    data: buf.slice(3..buf.len()),
                };
            }
            4 => {
                return Packet::AckPacket {
                    opcode: Opcode::ACK,
                    block_no: ((buf[1] as u16) << 8) | buf[2] as u16,
                };
            }
            5 => {
                return Packet::ErrPacket {
                    opcode: Opcode::ERR,
                    err_code: 0,
                    err_msg: "File not found".to_string(),
                };
            }
            _ => {
                panic!("Opcode error")
            }
        }
    }

    fn extract_str(arr: Bytes) -> String {
        let mut s = String::new();
        for i in 1..arr.len() {
            if arr[i].to_string().eq("0") {
                s = String::from_utf8(arr.slice(1..i).to_vec()).unwrap();
                return s;
            }
        }
        panic!("No EOF") // if no end-of-file 0 char found
    }
}