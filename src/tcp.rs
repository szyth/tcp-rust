use etherparse::TcpHeaderSlice;
use tun_tap::Iface;

pub enum State {
    Closed,
    Listen,
    // SynRcvd,
    // Estab,
}

impl Default for State {
    fn default() -> Self {
        State::Listen
    }
}

impl State {
    pub fn on_packet(
        &mut self,
        nic: &mut Iface,
        ip_header: etherparse::Ipv4HeaderSlice,
        tcp_header: TcpHeaderSlice,
        data: &[u8],
    ) -> Result<usize, std::io::Error> {
        let mut buf = [0u8; 1500];

        match *self {
            // if the State is Closed, then return nothing
            Self::Closed => return Ok(0),
            Self::Listen => {
                // expected SYN from client, if not return nothing
                if !tcp_header.syn() {
                    return Ok(0);
                }

                // now construct the SYN-ACK packet header that is sent from the server
                // after receiving Client Hello (SYN) to establish the connection
                let mut syn_ack = etherparse::TcpHeader::new(
                    tcp_header.destination_port(), // reverse the order of ports as SERVER will send the header
                    tcp_header.source_port(), // reverse the order of ports as SERVER will send the header
                    unimplemented!(),
                    unimplemented!(),
                );

                // set SYN in TcpHeader
                syn_ack.syn = true;

                // set ACK in TcpHeader
                syn_ack.ack = true;

                // now construct the IPv4 header that wraps up the TCP header, to be sent from server.
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    6, // refer: etherparse::IpNumber::Tcp,
                    [
                        ip_header.destination()[0],
                        ip_header.destination()[1],
                        ip_header.destination()[2],
                        ip_header.destination()[3],
                    ],
                    [
                        ip_header.source()[0],
                        ip_header.source()[1],
                        ip_header.source()[2],
                        ip_header.source()[3],
                    ],
                );

                // write out the headers
                // note: this part is a bit unclear to me
                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    let _ = ip.write(&mut unwritten);
                    let _ = syn_ack.write(&mut unwritten);
                    unwritten.len()
                };

                // send the (SYN-ACK + IP) packet to the Client
                return nic.send(&buf[..unwritten]);
            }
        }
        // let ip_src = ip_header.source_addr();
        // let ip_dst = ip_header.destination_addr();

        // eprintln!(
        //     "{}:{} -> {}:{} {}bytes of tcp",
        //     ip_src,
        //     tcp_header.source_port(),
        //     ip_dst,
        //     tcp_header.destination_port(),
        //     data.len()
        // );

        // Ok(0)
    }
}
