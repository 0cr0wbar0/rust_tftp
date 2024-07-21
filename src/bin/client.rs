use std::io;
use std::net::*;
use tftp::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:9000").unwrap();
    socket.connect("0.0.0.0:8000").expect("Couldn't connect to server");
    let mut request = String::new();
    println!("{}", "Select option: \n 1. send \n 2. receive");
    io::stdin().read_line(&mut request).unwrap();
    match request.trim().parse::<i32>().unwrap() {
        1 => {
            let mut write_packet = Packet::WrqPacket {
                opcode: Opcode::WRQ,
                filename: "write_test.txt".to_string(),
                mode: Mode::Octet,
            };
            write_packet.send(&socket);
            (write_packet, _) = Packet::receive(&socket);
            dbg!(&write_packet); // todo!("Find a way to print packets")
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
            todo!()
        }
    }
}
