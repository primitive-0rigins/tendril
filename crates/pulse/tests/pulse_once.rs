use serde_json::Value;
use std::net::UdpSocket;
use std::process::Command;
use std::time::Duration;

#[test]
fn pulse_once_sends_announcement_to_target() {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("bind UDP receiver");
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set timeout");
    let target = socket.local_addr().expect("read local addr");

    let output = Command::new(env!("CARGO_BIN_EXE_pulse"))
        .env("PULSE_ONCE", "1")
        .env("PULSE_NODE_NAME", "pulse-test")
        .env("PULSE_ADDR", "127.0.0.1:4100")
        .env("PULSE_TARGET", target.to_string())
        .output()
        .expect("run pulse once");

    assert!(output.status.success());

    let mut buf = [0_u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).expect("receive pulse announce");
    let message: Value = serde_json::from_slice(&buf[..len]).expect("pulse JSON");

    assert_eq!(message["type"], "pulse_announce");
    assert_eq!(message["node_name"], "pulse-test");
    assert_eq!(message["addr"], "127.0.0.1:4100");
}
