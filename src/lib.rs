use std::cmp;
use bytes::{BufMut, BytesMut};
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
        data: Vec<u8>,
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

const MAX_BUF_SIZE: usize = 516;
const MAX_DATA_SIZE: usize = MAX_BUF_SIZE - 4;

impl Packet {
    pub fn send(&self, socket: &UdpSocket) {
        // borrowing self in case need to use packet later
        // therefore, most attributes in packet need dereferencing
        let mut buf = BytesMut::with_capacity(MAX_BUF_SIZE);
        match self {
            Packet::WrqPacket {
                opcode, filename, ..
            } => {
                // EOF bytes in hex format
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
                buf.put(filename.as_bytes());
                buf.put_u8(0x00);
                buf.put(&b"octet"[..]);
                buf.put_u8(0x00);
            }
            Packet::RrqPacket {
                opcode, filename, ..
            } => {
                buf.put_u8(0);
                buf.put_u8(*opcode as u8);
                buf.put(filename.as_bytes());
                buf.put_u8(0x00);
                buf.put(&b"octet"[..]);
                buf.put_u8(0x00);
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
                buf.put_u8(0x00);
            }
        }
        socket.send(&buf).unwrap();
    }
    pub fn receive(socket: &UdpSocket) -> (Self, SocketAddr) {
        loop {
            let mut buf: Vec<u8> = vec![0; MAX_BUF_SIZE];
            if let Ok((size, src)) = socket.recv_from(&mut buf) {
                buf.truncate(size);
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
                            data: buf[4..buf.len()].to_owned(),
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

    fn extract_str(arr: Vec<u8>) -> String {
        for i in 2..arr.len() {
            if arr[i] == 0x00 {
                return String::from_utf8(arr[2..i].to_owned()).unwrap();
            }
        }
        panic!("No filename EOF") // if no end-of-file 0 char found
    }
    

    fn extract_err_msg(arr: Vec<u8>) -> String {
        for i in 4..arr.len() {
            if arr[i] == 0x00 {
                return String::from_utf8(arr[2..i].to_owned()).unwrap();
            }
        }
        panic!("No err msg EOF")
    }

    pub fn send_file(file_bytes: Vec<u8>, socket: &UdpSocket) -> Result<Vec<u8>, String> {
        let num_of_blocks = file_bytes.len() / MAX_DATA_SIZE + 1;
        for i in 1..=num_of_blocks {
            let start = (i - 1) * MAX_DATA_SIZE;
            let end = cmp::min(i * MAX_DATA_SIZE, file_bytes.len());
            let data = file_bytes[start..end].to_owned();
            let sent = Packet::DataPacket {
                opcode: Opcode::DATA,
                block_no: i as u16,
                data
            };
            sent.send(socket);
            let (received, _) = Packet::receive(socket);
            if let Packet::AckPacket {opcode: Opcode::ACK, block_no: num} = received {
                if num != i as u16 {
                    eprintln!("Packets received in wrong order!")
                }
            } else {
                return Err(String::from("Wrong packet type received!"));
            }
        }
        Ok(file_bytes)
    }

    pub fn receive_file(socket: &UdpSocket) -> Vec<u8> {
        let mut file_bytes: Vec<u8> = vec![];
        let mut reached_end: bool = false;
        loop {
            let (received, _) = Packet::receive(socket);
            if let Packet::DataPacket {
                opcode:Opcode::DATA,
                block_no: num,
                data: mut packet_bytes
            } = received {
                reached_end = packet_bytes.len() < MAX_DATA_SIZE;
                file_bytes.append(&mut packet_bytes);
                let sent = Packet::AckPacket {
                    opcode: Opcode::ACK,
                    block_no: num,
                };
                sent.send(socket);
            }
            if reached_end { break; }
        }
        file_bytes
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
        let mut buf = BytesMut::with_capacity(MAX_DATA_SIZE);
        buf.put_u8(1_u8);
        buf.put(&b"Hello!"[..]);
        buf.put_u8(0);

        let buf: Vec<u8> = buf.into();

        assert_eq!(Packet::extract_str(buf), String::from("Hello!"));
    }

    /// erroneous test to ensure proper EOF error:
    /// constructs byte array without a '0' character
    /// that indicates end of file name
    #[test]
    #[should_panic(expected = "No EOF")]
    fn fail_extract_str() {
        let mut buf = BytesMut::with_capacity(MAX_DATA_SIZE);
        buf.put_u8(1_u8);
        buf.put(&b"Hello!"[..]);

        let buf: Vec<u8> = buf.into();

        let _ = Packet::extract_str(buf);
    }

    /// erroneous test to ensure proper detection of invalid UTF-8:
    /// constructs gibberish byte array not in convertible format
    #[test]
    #[should_panic(expected = "FromUtf8Error")]
    fn malformed_utf8_extract_str() {
        let mut buf = BytesMut::with_capacity(MAX_DATA_SIZE);
        buf.put_u8(1_u8);
        buf.put(&[1_u8, 159, 146, 150][..]);
        buf.put_u8(0);

        let buf: Vec<u8> = buf.into();

        let _ = Packet::extract_str(buf);
    }
}
