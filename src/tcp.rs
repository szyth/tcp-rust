use etherparse::TcpHeaderSlice;
use tun_tap::Iface;

enum State {
    Closed,
    Listen,
    SynRcvd,
    // Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}

/// State of Send Sequence Space (RFC 793 S3.2 Fig 4)
///```
/// 1         2          3          4
/// ----------|----------|----------|----------
///   SND.UNA    SND.NXT    SND.UNA
///                        +SND.WND

/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers of unacknowledged data
/// 3 - sequence numbers allowed for new data transmission
/// 4 - future sequence numbers which are not yet allowed
///```
struct SendSequenceSpace {
    // send unacknowledged
    una: u32,
    // send next
    nxt: u32,
    // send window
    wnd: u16,
    // send urgent pointer
    up: bool,
    // segment sequence number used for last window update
    wl1: usize,
    // segment acknowledgment number used for last window update
    wl2: usize,
    // initial send sequence number
    iss: u32,
}

/// State of Receive Sequence Space (RFC 793 S3.2 Fig 5)
/// ```
/// 1          2          3
/// ----------|----------|----------
///    RCV.NXT    RCV.NXT
///              +RCV.WND

/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers allowed for new reception
/// 3 - future sequence numbers which are not yet allowed
/// ```
struct RecvSequenceSpace {
    // receive next
    nxt: u32,
    // receive window
    wnd: u16,
    // receive urgent pointer
    up: bool,
    // initial receive sequence number
    irs: u32,
}

impl Connection {
    pub fn accept(
        nic: &mut Iface,
        ip_header: etherparse::Ipv4HeaderSlice,
        tcp_header: TcpHeaderSlice,
        data: &[u8],
    ) -> Result<Option<Self>, std::io::Error> {
        let mut buf = [0u8; 1500];

        // expected SYN from client, if not return nothing
        if !tcp_header.syn() {
            return Ok(None);
        }

        let iss = 0; // any random number for sequence number
        let mut conn = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                // values to be sent from Server to the client
                iss: iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,

                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                // keep track on Client info
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                irs: tcp_header.sequence_number(),
                up: false,
            },
        };

        // after receiving Client Hello (SYN) construct the SYN-ACK packet header
        // that is sent from the server to establish the connection
        let mut syn_ack = etherparse::TcpHeader::new(
            tcp_header.destination_port(), // reverse the order of ports received from Client TCP Header
            tcp_header.source_port(), // reverse the order of ports received from Client TCP Header
            conn.send.iss,
            conn.send.wnd,
        );

        // set SYN in TcpHeader
        syn_ack.syn = true;

        // set ACK in TcpHeader
        syn_ack.ack = true;

        // set ACK Number for the SYN received from Client
        syn_ack.acknowledgment_number = conn.recv.nxt;

        // now construct the IPv4 header that wraps up the TCP header, to be sent from server.
        let mut ip = etherparse::Ipv4Header::new(
            syn_ack.header_len(),
            64,
            6, // refer: etherparse::IpNumber::Tcp,
            [
                // reverse the order of IP received from Client IP Header
                ip_header.destination()[0],
                ip_header.destination()[1],
                ip_header.destination()[2],
                ip_header.destination()[3],
            ],
            [
                // reverse the order of IP received from Client IP Header
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
        nic.send(&buf[..unwritten]);

        Ok(Some(conn))
    }

    pub fn on_packet(
        &mut self,
        nic: &mut Iface,
        ip_header: etherparse::Ipv4HeaderSlice,
        tcp_header: TcpHeaderSlice,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        Ok(())
    }
}
