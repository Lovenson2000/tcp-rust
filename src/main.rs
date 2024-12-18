use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    source: (Ipv4Addr, u16),
    destination: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let  mut connections :HashMap<Quad, tcp::State> =Default::default();  
    let mut network_interface = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun)?;
    
    let mut buffer = [0u8; 1504];
    loop {
        let number_of_bytes = network_interface.recv(&mut buffer[..])?;
        let _ethernet_flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let ethernet_protocol = u16::from_be_bytes([buffer[2], buffer[3]]);

        if ethernet_protocol != 0x0800 {
            continue;
        }
        match  etherparse::Ipv4HeaderSlice::from_slice(&buffer[4..number_of_bytes]) {
            Ok(ip_header) => {

                let source = ip_header.source_addr();
                let destination = ip_header.destination_addr();
                if ip_header.protocol() != 0x06 {
                    continue;
                }

                match  etherparse::TcpHeaderSlice::from_slice(&buffer[ 4 + ip_header.slice().len()..]) {
                    Ok(tcp_header) => {
                        let data_start_index = 4 + ip_header.slice().len() + tcp_header.slice().len();
                        connections.entry(Quad {
                            source: (source, tcp_header.source_port()),
                            destination: (destination, tcp_header.destination_port())
                        })
                        .or_default()
                        .on_packet(&mut network_interface, ip_header, tcp_header, &buffer[data_start_index..number_of_bytes]);
                    }
                    Err(e) => {
                        eprintln!("Ignoring tcp packet...{:?}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("Ignoring packet...{:?}", e)
            }
        }
    }
}
