use std::io;

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
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
    una: u32, // send unacknowledged
    nxt: u32, // send next
    wnd: u16, // send window
    up: bool, // send urgent pointer
    wl1: usize, // send segment sequence number used for last window update
    wl2: usize, // send segment acknowledgment  number used for last window
    iss: u32, // send initial send sequence number
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
    nxt: u32, // receive next
    wnd: u16, // receive window
    up: bool, // receive urgent pointer
    irs: u32, // receive initial receive sequence number

}


impl Connection {
    pub fn  accept<'a>(
        network_interface: &mut tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8], 
    ) -> io::Result<Option<Self>>{
        let mut buffer = [0u8; 1500];
        if !tcp_header.syn() {
            return Ok(None);
        }

        let iss = 0;

        let mut connecion = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss+ 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: ReceiveSequenceSpace {
                irs: tcp_header.sequence_number(),
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                up: false,
            }
        };

          // Establishing a conection
          let mut syn_ack = etherparse::TcpHeader::new(tcp_header.destination_port(), tcp_header.source_port(), connecion.send.iss, connecion.send.wnd);
          syn_ack.acknowledgment_number = connecion.recv.nxt;
          
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

          eprintln!("ip header: \n{:02x?}", ip_header);
          eprintln!("tcp header: \n{:02x?}", tcp_header);

          let unwritten = {
              let mut unwritten = &mut buffer[..];
              ip.write(&mut unwritten);
              syn_ack.write(&mut unwritten);
              unwritten.len()
          };

          eprintln!("Responding with {:02x?}", &buffer[..buffer.len() - unwritten]);
          network_interface.send(&buffer[..unwritten])?;
          Ok(Some(connecion))
    }


    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<()> {
        unimplemented!();
    }

}
