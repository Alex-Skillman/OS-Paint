use crate::input::{erase_at, stroke_erase_at};
use crate::stroke::Stroke;
use macroquad::color::Color;
use matchbox_socket::PeerId;
use matchbox_socket::{PeerState, WebRtcSocket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    pub layer: Option<i8>,
}

#[derive(Serialize, Deserialize)]
pub struct ErasePacket {
    pub point: (u16, u16),
    pub size: u16,
}

#[derive(Serialize, Deserialize)]
pub struct StrokeErasePacket {
    pub point: (u16, u16),
    pub size: u16,
}

#[derive(Serialize, Deserialize)]
pub struct StrokePacket {
    pub size: u16,
    pub color: SerColor,
    pub layer: i8,
    pub coordinates: Vec<(u16, u16)>,
}

#[derive(Serialize, Deserialize)]
pub struct CanvasSnapshotPacket {
    pub revision: u64,
    pub strokes: Vec<StrokePacket>,
    pub background: SerColor,
}

#[derive(Serialize, Deserialize)]
pub struct CursorPacket {
    pub point: (u16, u16),
    pub color: SerColor,
    pub name: String,
}

// A peer's last-known cursor position, the color they currently have
// selected, and their display name, so their indicator on the canvas (and
// their entry in the lobby list) shows who they are and what they're about
// to draw with.
#[derive(Clone)]
pub struct PeerCursor {
    pub point: (u16, u16),
    pub color: Color,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub enum NetworkPacket {
    Draw(DrawPacket),
    Erase(ErasePacket),
    StrokeErase(StrokeErasePacket),
    CanvasSnapshot(CanvasSnapshotPacket),
    Cursor(CursorPacket),
}

// A stable color derived from a peer's id (rather than anything they choose),
// so their cursor and their circle in the lobby list always match and stay
// consistent across everyone's screens without any coordination.
pub fn peer_color(peer: PeerId) -> Color {
    let mut hasher = DefaultHasher::new();
    peer.hash(&mut hasher);
    let hue = ((hasher.finish() >> 40) % 360) as f32;
    crate::ui::hsv_to_rgb(hue, 0.8, 0.95)
}

// A placeholder display name derived from a peer's id, shown until their
// actual name arrives with their first cursor update.
pub fn peer_label(peer: PeerId) -> String {
    let mut hasher = DefaultHasher::new();
    peer.hash(&mut hasher);
    format!("Guest-{:04X}", (hasher.finish() & 0xFFFF) as u16)
}

impl From<Color> for SerColor {
    fn from(c: Color) -> Self {
        SerColor {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}
impl From<SerColor> for Color {
    fn from(c: SerColor) -> Self {
        Color::new(c.r, c.g, c.b, c.a)
    }
}

impl From<&Stroke> for StrokePacket {
    fn from(stroke: &Stroke) -> Self {
        StrokePacket {
            size: stroke.size,
            color: stroke.color.into(),
            layer: stroke.layer,
            coordinates: stroke.coordinates.clone(),
        }
    }
}

impl From<StrokePacket> for Stroke {
    fn from(stroke: StrokePacket) -> Self {
        Stroke {
            size: stroke.size,
            color: stroke.color.into(),
            layer: stroke.layer,
            coordinates: stroke.coordinates,
        }
    }
}

pub fn peer_state(
    socket: &mut WebRtcSocket,
    strokes: &[Stroke],
    canvas_revision: u64,
    background: Color,
    peer_cursors: &mut HashMap<PeerId, PeerCursor>,
) {
    for (peer, state) in socket.update_peers() {
        match state {
            PeerState::Connected => {
                println!("Peer Joined");
                send_canvas_snapshot_to_peer(socket, strokes, canvas_revision, background, peer);
            }
            PeerState::Disconnected => {
                println!("Peer Left");
                peer_cursors.remove(&peer);
            }
        }
    }
}

pub fn send_cursor(socket: &mut WebRtcSocket, point: (u16, u16), color: Color, name: &str) {
    send_packet(
        socket,
        &NetworkPacket::Cursor(CursorPacket { point, color: color.into(), name: name.to_string() }),
    );
}

pub fn send_packet(socket: &mut WebRtcSocket, packet: &NetworkPacket) {
    let bytes = bincode::serialize(packet).unwrap();

    for peer in socket.connected_peers().collect::<Vec<_>>() {
        socket
            .channel_mut(0)
            .send(bytes.clone().into_boxed_slice(), peer);
    }
}

fn canvas_snapshot_packet(strokes: &[Stroke], canvas_revision: u64, background: Color) -> NetworkPacket {
    NetworkPacket::CanvasSnapshot(CanvasSnapshotPacket {
        revision: canvas_revision,
        strokes: strokes.iter().map(StrokePacket::from).collect(),
        background: background.into(),
    })
}

pub fn send_canvas_snapshot(socket: &mut WebRtcSocket, strokes: &[Stroke], canvas_revision: u64, background: Color) {
    send_packet(socket, &canvas_snapshot_packet(strokes, canvas_revision, background));
}

fn send_canvas_snapshot_to_peer(
    socket: &mut WebRtcSocket,
    strokes: &[Stroke],
    canvas_revision: u64,
    background: Color,
    peer: PeerId,
) {
    let bytes = bincode::serialize(&canvas_snapshot_packet(strokes, canvas_revision, background)).unwrap();
    socket.channel_mut(0).send(bytes.into_boxed_slice(), peer);
}

pub fn handle_incoming(
    socket: &mut WebRtcSocket,
    strokes: &mut Vec<Stroke>,
    peer_current_stroke: &mut HashMap<PeerId, usize>,
    canvas_revision: &mut u64,
    background: &mut Color,
    local_stroke_idx: &mut Option<usize>,
    peer_cursors: &mut HashMap<PeerId, PeerCursor>,
) {
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
                    peer_current_stroke.insert(peer, strokes.iter().len() - 1);
                } else if let Some(&idx) = peer_current_stroke.get(&peer) {
                    strokes[idx].coordinates.push(packet.point);
                }
                *canvas_revision += 1;
            }
            Ok(NetworkPacket::Erase(packet)) => {
                if erase_at(strokes, packet.point, packet.size) {
                    *canvas_revision += 1;
                    peer_current_stroke.clear();
                    *local_stroke_idx = None;
                }
            }
            Ok(NetworkPacket::StrokeErase(packet)) => {
                if stroke_erase_at(strokes, packet.point, packet.size) {
                    *canvas_revision += 1;
                    peer_current_stroke.clear();
                    *local_stroke_idx = None;
                }
            }
            Ok(NetworkPacket::CanvasSnapshot(packet)) => {
                if packet.revision > *canvas_revision {
                    *strokes = packet.strokes.into_iter().map(Stroke::from).collect();
                    *canvas_revision = packet.revision;
                    *background = packet.background.into();
                    peer_current_stroke.clear();
                    *local_stroke_idx = None;
                }
            }
            Ok(NetworkPacket::Cursor(packet)) => {
                peer_cursors.insert(
                    peer,
                    PeerCursor { point: packet.point, color: packet.color.into(), name: packet.name },
                );
            }
            Err(_e) => eprintln!("Bad packet"),
        }
    }
}
