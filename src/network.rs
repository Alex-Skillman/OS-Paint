use matchbox_socket::{WebRtcSocket, PeerState};
use macroquad::color::Color;
use serde::{Serialize, Deserialize};
use bincode::Serialize;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SerColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Serialize, Deserialize)]
struct DrawPacker {
    point: (u16, u16),
    is_new_stroke: bool,
    size: Option<u16>,
    color: Option<SerColor>,
    layer: Option<i8>
}

impl From<Color> for SerColor {
    fn from(c: Color) -> Self {
        SerColor { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}
impl From<SerColor> for Color {
    fn from(c: SerColor) -> Self {
        Color::new(c.r, c.g, c.b, c.a)
    }
}

pub fn peer_state(socket: &mut WebRtcSocket) {
    for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => println!("Peer Joined"),
                PeerState::Disconnected => println!("Peer Left"),
            }
        }
}

pub fn send_packet(socket: &mut WebRtcSocket, packet: &DrawPacket) {
    let bytes = bincode::serialize(packet).unwrap();

    for peer in socket.connected_peers().collect::<Vec<_>>() {
        socket.channel_mut(0).send(bytes.clone().into_boxed_slice(), peer);
    }
}
pub fn handle_incoming(socket: &mut WebRtcSocket,) {
    for (peer, packet_bytes) in socket.channel_mut(0).receive() {
        // DESERIALIZE HERE
    }
}