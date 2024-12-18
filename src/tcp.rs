use std::io;

enum State {
    Closed,
    Listen,
    // synRcvd,
    // Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: ReceiveSequenceSpace,
}

// State of the Send Sequence Space (RFC 793 S3.2 F4)

//                    1         2          3          4
//               ----------|----------|----------|----------
//                      SND.UNA    SND.NXT    SND.UNA
//                                           +SND.WND

//         1 - old sequence numbers which have been acknowledged
//         2 - sequence numbers of unacknowledged data
//         3 - sequence numbers allowed for new data transmission
//         4 - future sequence numbers which are not yet allowed

//                           Send Sequence Space


struct SendSequenceSpace {
    una: usize, // send unacknowledged
    nxt: usize, // send next
    wnd: usize, // send window
    up: bool, // send urgent pointer
    wl1: usize, // send segment sequence number used for last window update
    wl2: usize, // send segment acknowledgment  number used for last window
    iss: usize, // send initial send sequence number
}


// State of the Receive Sequence Space (RFC 793 S3.2 F4)
//  1          2          3
//        ----------|----------|----------
//             RCV.NXT    RCV.NXT
//                 +RCV.WND

// 1 - old sequence numbers which have been acknowledged
// 2 - sequence numbers allowed for new reception
// 3 - future sequence numbers which are not yet allowed

//                     Receive Sequence Space

struct ReceiveSequenceSpace {
    nxt: usize, // receive next
    wnd: usize, // receive window
    up: bool, // receive urgent pointer
    irs: usize, // receive initial receive sequence number

}

impl Default for Connection {
    fn default() -> Self {
        //State::Closed;
        Connection {
            state: State::Listen,
        }
    }
}

impl State {
    pub fn  on_packet<'a>(
        &mut self,
        network_interface: &mut tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<usize>{
        let mut buffer = [0u8; 1500];
        match self.state {
            State::Closed => {
                return Ok(0);
            }
            State::Listen => {
                if !tcp_header.syn() {
                    return Ok(0);
                }

                // Keep track of sender data
                self.recv.nxt = tcp_header.sequence_number() + 1;
                self.recv.wnd = tcp_header.window_size();
                self.recv.irs = tcp_header.sequence_number();

                // Establishing a conection
                let mut syn_ack = etherparse::TcpHeader::new(tcp_header.destination_port(), tcp_header.source_port(), 0, 10);
                syn_ack.acknowledgment_number = self.recv.nxt;
                
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    etherparse::IpTrafficClass::Tcp,
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

                // Write out the headers

                let unwritten = {
                    let mut unwritten = &mut buffer[..];
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };
                network_interface.send(&buffer[..unwritten]);
            }
        }
    }
}