//!
//!# Very Simple Websocket For Rust
//!
//!`ws2` is a very easy to use WebSocket server & client for Rust, builds on `ws` crate. it was designed for production, the receive timeout provides a non-blocking way.
//!
//!## Server Example
//!
//!### Server with simple handler
//!
//!
//!```rust
//!use log2::*;
//!use ws2::{server::*, WebSocket};
//!
//!struct Worker;
//!
//!impl ws2::Handler for Worker {
//!    fn on_open(&self, ws: &WebSocket) -> Pod {
//!        info!("on open: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_close(&self, ws: &WebSocket) -> Pod {
//!        info!("on close: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_message(&self, ws: &WebSocket, msg: String) -> Pod {
//!        info!("on message: {msg}, {ws}");
//!        let echo = format!("echo: {msg}");
//!        let n = ws.send(echo);
//!        Ok(n?)
//!    }
//!}
//!
//!fn main() -> anyhow::Result<()> {
//!    let _log2 = log2::start();
//!    let mut server = ws2::server::listen("127.0.0.1:3125")?;
//!    let worker = Worker {};
//!
//!    loop {
//!        let _ = server.process(&worker, 0.5);
//!        // do other stuff
//!        // ...
//!    }
//!}
//!
//!```
//!
//!### Server with simple events
//!
//!```rust
//!use log2::*;
//!use std::collections::HashMap;
//!use ws2::server::*;
//!
//!fn main() -> anyhow::Result<()> {
//!    let _log2 = log2::start();
//!    let server = ws2::server::listen("127.0.0.1:3125")?;
//!    let mut n = 0;
//!    let mut map = HashMap::new();
//!
//!    loop {
//!        match server.recv(0.5) {
//!            Event::Open(id, sender, address) => {
//!                debug!("{id} on open: {address}");
//!                let _ = sender.send("Hello, client");
//!                map.insert(id, sender);
//!            }
//!            Event::Close(id) => {
//!                debug!("{id} on close");
//!                map.remove(&id);
//!            }
//!            Event::Text(id, message) => {
//!                debug!("{id} on message: {message}");
//!                let ws = &map[&id];
//!                let _ = ws.send(format!("echo {message}"));
//!            }
//!            Event::Binary(id, _) => {
//!                debug!("{id} on binary");
//!            }
//!            Event::Timeout => {
//!                // debug!("timeout");
//!            }
//!            Event::Error(e) => {
//!                debug!("on error: {e}");
//!                n += 1;
//!                if n >= 10 {
//!                    break;
//!                }
//!            }
//!        }
//!    }
//!
//!    Ok(())
//!}
//!
//!```
//!
//!## Client Example
//!
//!### Client with simple handler
//!
//!```rust
//!use log2::*;
//!use ws2::{client::*, Pod, WebSocket};
//!
//!struct Worker;
//!
//!impl ws2::Handler for Worker {
//!    fn on_open(&self, ws: &WebSocket) -> Pod {
//!        // ws.send("Hello World")?;
//!        info!("on open: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_close(&self, ws: &WebSocket) -> Pod {
//!        info!("on close: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_message(&self, ws: &WebSocket, msg: String) -> Pod {
//!        info!("on message: {msg}, {ws}");
//!        Ok(())
//!    }
//!}
//!
//!fn main() -> anyhow::Result<()> {
//!    let _log2 = log2::start();
//!    let url = "wss://stream.binance.com:9443/ws/btcusdt@miniTicker";
//!    let mut ws = ws2::client::connect(url)?;
//!    let handler = Worker {};
//!
//!    loop {
//!        let _ = ws.process(&handler, 0.5);
//!    }
//!}
//!
//!```
//!
//!### Client with simple events
//!
//!The websocket client was designed as out-of-the-box, it will auto reconnect every 3s.
//!
//!```rust
//!use log2::*;
//!use ws2::client::*;
//!
//!fn main() -> anyhow::Result<()> {
//!    let _log2 = log2::start();
//!    let url = "wss://stream.binance.com:9443/ws/btcusdt@miniTicker";
//!
//!    let mut ws = ws2::client::connect(url)?;
//!    let mut n = 0;
//!
//!    loop {
//!        match ws.recv(0.5) {
//!            Event::Open(_) => {
//!                info!("on open");
//!            }
//!            Event::Close => {
//!                info!("on close");
//!            }
//!            Event::Text(msg) => {
//!                info!("on message: {msg}");
//!            }
//!            Event::Binary(_) => {}
//!            Event::Timeout => {
//!                info!("timeout");
//!            }
//!            Event::Error(e) => {
//!                error!("error: {e}");
//!                n += 1;
//!                if n >= 10 {
//!                    break;
//!                }
//!            }
//!        }
//!    }
//!
//!    Ok(())
//!}
//!
//!```
//!
pub mod client;
pub mod server;

use std::fmt::Display;
use ws::{Error, Message, Result, Sender};

/// Flexible result
pub type Pod = anyhow::Result<(), anyhow::Error>;

pub struct WebSocket {
    address: String,
    id: u32,
    sender: Sender,
}

impl WebSocket {
    #[inline]
    pub fn send<M>(&self, msg: M) -> Result<()>
    where
        M: Into<Message>,
    {
        self.sender.send(msg)
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn close(&self) -> Pod {
        let n = self.sender.close(ws::CloseCode::Normal);
        Ok(n?)
    }
}

impl Display for WebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[allow(unused)]
pub trait Handler {
    fn on_open(&self, ws: &WebSocket) -> Pod {
        Ok(())
    }

    fn on_close(&self, ws: &WebSocket) -> Pod {
        Ok(())
    }

    fn on_message(&self, ws: &WebSocket, msg: String) -> Pod {
        Ok(())
    }

    fn on_binary(&self, ws: &WebSocket, buf: Vec<u8>) -> Pod {
        Ok(())
    }

    fn on_error(&self, error: Error) -> Pod {
        Ok(())
    }

    fn on_timeout(&self) -> Pod {
        Ok(())
    }
}
