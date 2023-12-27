use std::{
    collections::{hash_map::Entry, HashMap},
    net::Ipv4Addr,
};

use tun_tap::Iface;
mod tcp;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> Result<(), std::io::Error> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut nic = Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;

        let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);

        // Ignore Packets if not ipv4 (0x0800)
        // https://en.wikipedia.org/wiki/EtherType
        if eth_proto != 0x0800 {
            // not ipv4
            continue;
        }

        // TUNTAP interface has 2bytes for flags, and 2bytes for proto
        // 3.2 Frame format: https://www.kernel.org/doc/Documentation/networking/tuntap.txt
        let eth_header_size = 4;

        // parsing IPv4 Packet Header

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[eth_header_size..nbytes]) {
            // end range is `nbytes` because `buf` has 1504 length
            // but only the bytes received from `nic.recv()` is what we need, which is in `nbytes`
            Ok(ip_header) => {
                let ip_src = ip_header.source_addr();
                let ip_dst = ip_header.destination_addr();
                let _len = ip_header.payload_len();
                let proto = ip_header.protocol();

                if proto != 0x06 {
                    // not TCP
                    continue;
                }

                // IPv4 Header Bytes size
                let ip_header_size = ip_header.slice().len();

                // PARSING TCP HEADER

                // start index of TCP header Bytes starts after ethheader and ipheader
                // end range is `nbytes` because `buf` has 1504 length
                // but only the bytes received from `nic.recv()` is what we need, which is upto `nbytes` length
                match etherparse::TcpHeaderSlice::from_slice(
                    &buf[eth_header_size + ip_header_size..nbytes],
                ) {
                    Ok(tcp_header) => {
                        // TCP Header Bytes size
                        let tcp_header_size = tcp_header.slice().len();

                        // start index of TCP Data/Payload after eth_header, ip_header and tcp_header
                        let data_index = eth_header_size + ip_header_size + tcp_header_size;

                        match connections.entry(Quad {
                            src: (ip_src, tcp_header.source_port()),
                            dst: (ip_dst, tcp_header.destination_port()),
                        }) {
                            Entry::Occupied(mut conn) => {
                                let _ = conn.get_mut().on_packet(
                                    &mut nic,
                                    ip_header,
                                    tcp_header,
                                    &buf[data_index..nbytes],
                                );
                            }
                            // accept a connection if a new Quad is received
                            Entry::Vacant(e) => {
                                if let Some(conn) = tcp::Connection::accept(
                                    &mut nic,
                                    ip_header,
                                    tcp_header,
                                    &buf[data_index..nbytes],
                                )? {
                                    e.insert(conn);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Ignoring packet: {}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("Ignoring packet: {}", e)
            }
        };
    }
}
