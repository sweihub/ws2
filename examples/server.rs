use log2::*;
use std::collections::HashMap;
use ws2::server::*;

fn main() -> anyhow::Result<()> {
    let _log2 = log2::start();
    let server = ws2::server::listen("127.0.0.1:3125")?;
    let mut n = 0;
    let mut map = HashMap::new();

    loop {
        match server.recv(0.5) {
            Event::Open(id, sender, address) => {
                debug!("{id} on open: {address}");
                let _ = sender.send("Hello, client");
                map.insert(id, sender);
            }
            Event::Close(id) => {
                debug!("{id} on close");
                map.remove(&id);
            }
            Event::Text(id, message) => {
                debug!("{id} on message: {message}");
                let ws = &map[&id];
                let _ = ws.send(format!("echo {message}"));
            }
            Event::Binary(id, _) => {
                debug!("{id} on binary");
            }
            Event::Timeout => {
                // debug!("timeout");
            }
            Event::Error(e) => {
                debug!("on error: {e}");
                n += 1;
                if n >= 10 {
                    break;
                }
            }
        }
    }

    Ok(())
}
