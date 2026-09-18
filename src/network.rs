use matchbox_socket::{WebRtcSocket, PeerState};
use macroquad::color::Color;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use matchbox_socket::PeerId;
use crate::stroke::Stroke;
use crate::input::erase_at;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SerColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Serialize, Deserialize)]
pub struct DrawPacket {
    pub point: (u16, u16),
    pub is_new_stroke: bool,
    pub size: Option<u16>,
    pub color: Option<SerColor>,
    pub layer: Option<i8>
}

#[derive(Serialize, Deserialize)]
pub struct ErasePacket {
    pub point: (u16, u16),
    pub size: u16,
}

#[derive(Serialize, Deserialize)]
pub enum NetworkPacket {
    Draw(DrawPacket),
    Erase(ErasePacket),
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
    for (_peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => println!("Peer Joined"),
                PeerState::Disconnected => println!("Peer Left"),
            }
        }
}

pub fn send_packet(socket: &mut WebRtcSocket, packet: &NetworkPacket) {
    let bytes = bincode::serialize(packet).unwrap();

    for peer in socket.connected_peers().collect::<Vec<_>>() {
        socket.channel_mut(0).send(bytes.clone().into_boxed_slice(), peer);
    }
}
pub fn handle_incoming(socket: &mut WebRtcSocket, strokes: &mut Vec<Stroke>, peer_current_stroke: &mut HashMap<PeerId, usize>) {
    for (peer, packet_bytes) in socket.channel_mut(0).receive() {
        match bincode::deserialize::<NetworkPacket>(&packet_bytes) {
            Ok(NetworkPacket::Draw(packet)) => {
                if packet.is_new_stroke {
                    strokes.push(Stroke {
                        size: packet.size.unwrap(),
                        color: packet.color.unwrap().into(),
                        layer: packet.layer.unwrap(),
                        coordinates: vec![packet.point],
                    });
                    peer_current_stroke.insert(peer,strokes.iter().len() -1);
                } else if let Some(&idx) = peer_current_stroke.get(&peer) {
                    strokes[idx].coordinates.push(packet.point);
                }
            }
            Ok(NetworkPacket::Erase(packet)) => {
                erase_at(strokes, packet.point, packet.size);
                peer_current_stroke.clear();
            }
            Err(_e) => eprintln!("Bad packet")
        }
    }
}
