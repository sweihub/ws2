//!
//!# Very Simple Websocket For Rust
//!
//!`ws2` is a very easy to use WebSocket server & client for Rust, builds on `ws` crate. it was designed for production, the receive timeout provides a non-blocking way.
//!
//!## Server Example
//!
//!```rust
//!use log2::*;
//!use ws2::{Pod, WebSocket};
//!
//!struct Worker;
//!
//!impl ws2::Handler for Worker {
//!    fn on_open(&mut self, ws: &WebSocket) -> Pod {
//!        info!("on open: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_close(&mut self, ws: &WebSocket) -> Pod {
//!        info!("on close: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_message(&mut self, ws: &WebSocket, msg: String) -> Pod {
//!        info!("on message: {msg}, {ws}");
//!        let echo = format!("echo: {msg}");
//!        let n = ws.send(echo);
//!        Ok(n?)
//!    }
//!}
//!
//!fn main() -> Pod {
//!    let _log2 = log2::start();
//!    let address = "127.0.0.1:3125";
//!    let mut worker = Worker {};
//!
//!    info!("listen on: {address}");
//!    let mut server = ws2::listen(address)?;
//!
//!    loop {
//!        let _ = server.process(&mut worker, 0.5);
//!        // do other stuff
//!    }
//!}
//!
//!```
//!
//!## Client Example
//!
//!The websocket client was designed as out-of-the-box, it will auto reconnect every 3s.
//!
//!```rust
//!use log2::*;
//!use ws2::{Pod, WebSocket};
//!
//!struct Worker;
//!
//!impl ws2::Handler for Worker {
//!    fn on_open(&mut self, ws: &WebSocket) -> Pod {
//!        // ws.send("Hello World")?;
//!        info!("on open: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_close(&mut self, ws: &WebSocket) -> Pod {
//!        info!("on close: {ws}");
//!        Ok(())
//!    }
//!
//!    fn on_message(&mut self, ws: &WebSocket, msg: String) -> Pod {
//!        info!("on message: {msg}, {ws}");
//!        Ok(())
//!    }
//!}
//!
//!fn main() -> Pod {
//!    let _log2 = log2::start();
//!    let url = "wss://stream.binance.com:9443/ws/btcusdt@miniTicker";
//!    let mut client = ws2::connect(url)?;
//!    let mut workder = Worker {};
//!
//!    loop {
//!        let _ = client.process(&mut workder, 0.5);
//!        // do other stuff
//!    }
//!}
//!```
//!
pub mod client;
pub mod server;

use std::fmt::Display;
use ws::{Error, Message, Result};

/// Flexible result
pub type Pod = anyhow::Result<(), anyhow::Error>;

/// Wait infinitely
pub const INFINITE: f32 = std::f32::MAX;

pub use client::connect;
pub use client::Client;
pub use server::listen;
pub use server::Server;
pub use ws::Sender;

#[derive(Clone)]
/// WebSocket sender
pub struct WebSocket {
    address: String,
    id: u32,
    sender: Sender,
}

impl WebSocket {
    #[inline]
    /// send string or binary
    pub fn send<M>(&self, msg: M) -> Result<()>
    where
        M: Into<Message>,
    {
        self.sender.send(msg)
    }

    /// peer address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// unique websocket id
    pub fn id(&self) -> u32 {
        self.id
    }

    /// close the websocket
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
    fn on_open(&mut self, ws: &WebSocket) -> Pod {
        Ok(())
    }

    fn on_close(&mut self, ws: &WebSocket) -> Pod {
        Ok(())
    }

    fn on_message(&mut self, ws: &WebSocket, msg: String) -> Pod {
        Ok(())
    }

    fn on_binary(&mut self, ws: &WebSocket, buf: Vec<u8>) -> Pod {
        Ok(())
    }

    fn on_error(&mut self, error: Error) -> Pod {
        Ok(())
    }

    fn on_timeout(&mut self) -> Pod {
        Ok(())
    }
}
