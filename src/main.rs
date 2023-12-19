use tun_tap::Iface;

fn main() -> Result<(), std::io::Error> {
    let nic = Iface::new("tun0", tun_tap::Mode::Tun)?;
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

        // parsing IPv4 Packet Header
        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            // range: [4..nbytes]
            // as TUNTAP interface has 2bytes for flags, and 2bytes for proto
            // 3.2 Frame format: https://www.kernel.org/doc/Documentation/networking/tuntap.txt
            Ok(ip_header) => {
                let src = ip_header.source_addr();
                let dst = ip_header.destination_addr();
                let _len = ip_header.payload_len();
                let proto = ip_header.protocol();

                if proto != 0x06 {
                    // not TCP
                    continue;
                }

                // parsing TCP Header
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + ip_header.slice().len()..]) {
                    // range: [4+ip_header.slice().len()..]
                    // 4: mentioned earlier
                    // `ip_header.slice().len()`: IPv4 Header Bytes
                    // remaining are the TCP header Bytes
                    Ok(tcp_header) => {
                        eprintln!(
                            "{} -> {} {}b of tcp to port {}",
                            src,
                            dst,
                            tcp_header.slice().len(),
                            tcp_header.destination_port()
                        );
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
