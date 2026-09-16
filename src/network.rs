use matchbox_socket::{WebRtcSocket, PeerState};

pub fn tokio_runtime() {
    // Create a tokio runtime inside the loop
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // This inits the address to a local signalling server
    // THIS WILL ONLY WORK LOCALLY RIGHT NOW
    let( socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/my_room");
}