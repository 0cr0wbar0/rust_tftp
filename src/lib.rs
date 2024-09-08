use bytes::{BufMut, Bytes, BytesMut};
use std::net::{SocketAddr, UdpSocket};

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Opcode {
    RRQ = 1,
    WRQ = 2,
    DATA = 3,
    ACK = 4,
    ERR = 5,
}
#[derive(Debug)]
pub enum Mode {
    Octet,
}

#[derive(Debug)]
pub enum Packet {
    WrqPacket {
        opcode: Opcode,
        filename: String,
        mode: Mode,
    },
    RrqPacket {
        opcode: Opcode,
        filename: String,
        mode: Mode,
    },
    DataPacket {
        opcode: Opcode,
        block_no: u16,
        data: Bytes,
    },
    AckPacket {
        opcode: Opcode,
        block_no: u16,
    },
    ErrPacket {
        opcode: Opcode,
        err_code: u16,
        err_msg: String,
    },
}

impl Packet {
    pub fn send(&self, socket: &UdpSocket) {
        // borrowing self in case need to use packet later
        // therefore, most attributes in packet need dereferencing
        let mut buf = BytesMut::with_capacity(512);
        match self {
            Packet::WrqPacket {
                opcode, filename, ..
            } => {
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
                buf.put(filename.as_bytes());
                buf.put_u8(0);
                buf.put(&b"octet"[..]);
                buf.put_u8(0);
            }
            Packet::RrqPacket {
                opcode, filename, ..
            } => {
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
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
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
                buf.put_u16(*block_no);
                buf.put(&data[..]);
            }
            Packet::AckPacket { opcode, block_no } => {
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
                buf.put_u16(*block_no);
            }
            Packet::ErrPacket {
                opcode,
                err_code,
                err_msg,
            } => {
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
                buf.put_u16(*err_code);
                buf.put(err_msg.as_bytes());
                buf.put_u8(0);
            }
        }
        socket.send(&buf).unwrap();
    }
    pub fn receive(socket: &UdpSocket) -> (Self, SocketAddr) {
        let mut received = vec![0; 512];
        loop {
            if let Ok((_, src)) = socket.recv_from(&mut received) {
                let buf = Bytes::from(received);
                return (match buf[1] {
                    1 => {
                        Packet::RrqPacket {
                            opcode: Opcode::RRQ,
                            filename: Self::extract_str(buf),
                            mode: Mode::Octet,
                        }
                    }
                    2 => {
                        Packet::WrqPacket {
                            opcode: Opcode::WRQ,
                            filename: Self::extract_str(buf),
                            mode: Mode::Octet,
                        }
                    }
                    3 => {
                        Packet::DataPacket {
                            opcode: Opcode::DATA,
                            // bitwise operation to convert two u8s into one u16, thanks stackoverflow :)
                            block_no: ((buf[2] as u16) << 8) | buf[3] as u16,
                            data: buf.slice(4..buf.len()),
                        }
                    }
                    4 => {
                        Packet::AckPacket {
                            opcode: Opcode::ACK,
                            block_no: ((buf[2] as u16) << 8) | buf[3] as u16,
                        }
                    }
                    5 => {
                        Packet::ErrPacket {
                            opcode: Opcode::ERR,
                            err_code: ((buf[2] as u16) << 8) | buf[3] as u16,
                            err_msg: Self::extract_err_msg(buf),
                        }
                    }
                    _ => {
                        panic!("Opcode error")
                    }
                }, src);
            }
        }
    }

    fn extract_str(arr: Bytes) -> String {
        for i in 2..arr.len() {
            if arr[i] == 0 {
                return String::from_utf8(arr.slice(2..i).to_vec()).unwrap();
            }
        }
        panic!("No filename EOF") // if no end-of-file 0 char found
    }

    fn extract_err_msg(arr:Bytes) -> String {
        for i in 4..arr.len() {
            if arr[i] == 0 {
                return String::from_utf8(arr.slice(4..i).to_vec()).unwrap();
            }
        }
        panic!("No err msg EOF")
    }
}

#[cfg(test)] // Only compiles if cargo test is executed
mod tests {
    // Inline definition of separate Rust file
    use super::*; // imports crates from outer file

    /// normal Rust test to ensure string is extracted properly:
    /// constructs byte array with '0' character to indicate
    /// end of file name
    #[test]
    fn test_extract_str() {
        let mut buf = BytesMut::with_capacity(512);
        buf.put_u8(1_u8);
        buf.put(Bytes::from(&b"Hello!"[..]));
        buf.put_u8(0);

        let buf: Bytes = buf.into();

        assert_eq!(Packet::extract_str(buf), String::from("Hello!"));
    }

    /// erroneous test to ensure proper EOF error:
    /// constructs byte array without a '0' character
    /// that indicates end of file name
    #[test]
    #[should_panic(expected = "No EOF")]
    fn fail_extract_str() {
        let mut buf = BytesMut::with_capacity(512);
        buf.put_u8(1_u8);
        buf.put(Bytes::from(&b"Hello!"[..]));

        let buf: Bytes = buf.into();

        let _ = Packet::extract_str(buf);
    }

    /// erroneous test to ensure proper detection of invalid UTF-8:
    /// constructs gibberish byte array not in convertible format
    #[test]
    #[should_panic(expected = "FromUtf8Error")]
    fn malformed_utf8_extract_str() {
        let mut buf = BytesMut::with_capacity(512);
        buf.put_u8(1_u8);
        buf.put(Bytes::from(&[1_u8, 159, 146, 150][..]));
        buf.put_u8(0);

        let buf: Bytes = buf.into();

        let _ = Packet::extract_str(buf);
    }
}
