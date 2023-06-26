use log2::*;
use std::collections::HashMap;
use ws2::{server::*, Pod, WebSocket};

#[allow(unused)]
fn main1() -> Pod {
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

struct Worker;

impl ws2::Handler for Worker {
    fn on_open(&self, ws: &WebSocket) -> Pod {
        info!("on open: {ws}");
        Ok(())
    }

    fn on_close(&self, ws: &WebSocket) -> Pod {
        info!("on close: {ws}");
        Ok(())
    }

    fn on_message(&self, ws: &WebSocket, msg: String) -> Pod {
        info!("on message: {msg}, {ws}");
        let echo = format!("echo: {msg}");
        let n = ws.send(echo);
        Ok(n?)
    }
}

fn main() -> Pod {
    let _log2 = log2::start();
    let mut server = ws2::server::listen("127.0.0.1:3125")?;
    let worker = Worker {};

    loop {
        let _ = server.process(&worker, 0.5);
        // do other stuff
    }
}
