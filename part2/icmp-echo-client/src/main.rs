#![feature(exclusive_range_pattern)]

use pnet::{
	transport::{TransportChannelType, TransportProtocol, transport_channel, icmpv6_packet_iter},
	packet::{
		Packet, 
		icmpv6::{Icmpv6Types::{EchoRequest, EchoReply}, Icmpv6Packet},
		ip::IpNextHeaderProtocols::Icmpv6
	}
};

const CODE_LEN: usize = 4;
const CODE: [u8; CODE_LEN] = [b'm', b'D', b'1', b'6'];
const ICMP_HEADER_LEN: usize = 4;
const ECHO_HEADER_LEN: usize = 4;

fn main() {
	let (mut tx, mut rx) = match transport_channel(4096, TransportChannelType::Layer4(TransportProtocol::Ipv6(Icmpv6))) {
		Ok((tx, rx)) => (tx, rx),
		Err(e) => panic!("Failed to create the channel: {}", e)
	};
	let mut rx_iter = icmpv6_packet_iter(&mut rx);
	let mut buf = [0u8; 65536];
	buf[0] = EchoReply.0;
	println!("Hongbao is ready.");
	loop {
		match rx_iter.next() {
			Ok((packet, addr)) => {
				if packet.get_icmpv6_type() != EchoRequest { continue; }
				let payload = packet.payload();
				if payload.len() < ECHO_HEADER_LEN { continue; }
				println!("Received an ICMP echo request from {} with ID and Sequence# {:02x?}", addr, &payload[..ECHO_HEADER_LEN]);

				let payload_len = payload.len();
				let buf_len = ECHO_HEADER_LEN + payload_len + CODE_LEN;
				buf[ICMP_HEADER_LEN..(ICMP_HEADER_LEN + payload_len)].copy_from_slice(&payload[..payload_len]);
				buf[(ICMP_HEADER_LEN + payload_len)..buf_len].copy_from_slice(&CODE);
				
				let new = Icmpv6Packet::new(&buf[..buf_len]).unwrap();
				match tx.send_to(new, addr) {
					Ok(n) => println!("Sent an ICMP echo reply with {} bytes payload to {}", n, addr),
					Err(e) => eprintln!("Failed to send reply to {}: {}", addr, e)
				}
			},
			Err(e) => panic!("Failed to receive a packet: {}", e)
		}
	}
}
