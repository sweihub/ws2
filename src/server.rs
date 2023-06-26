use crate::Handler;
use crate::Pod;
use crate::WebSocket;
pub use crate::INFINITE;
use log2::debug;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::channel;
use std::sync::mpsc::RecvTimeoutError::{Disconnected, Timeout};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use ws::{CloseCode, Error, ErrorKind, Handshake, Message, Result, Sender};

pub enum Event {
    Open(u32, Sender, String),
    Close(u32),
    Text(u32, String),
    Binary(u32, Vec<u8>),
    Timeout,
    Error(Error),
}

pub struct Server {
    rx: std::sync::mpsc::Receiver<Event>,
    radio: Sender,
    thread: Option<JoinHandle<Pod>>,
    map: HashMap<u32, WebSocket>,
}

impl Server {
    /// Receive the events, timeout is decimal seconds
    pub fn recv(&self, timeout: f32) -> Event {
        let mut n = Duration::from_nanos(std::u64::MAX);
        if timeout != INFINITE {
            n = Duration::from_nanos((timeout * 1e9) as u64);
        }
        match self.rx.recv_timeout(n) {
            Ok(event) => event,
            Err(e) => match e {
                Timeout => Event::Timeout,
                Disconnected => Event::Error(Error::new(ErrorKind::Internal, e.to_string())),
            },
        }
    }

    /// Process one event with decimal seconds timeout, or [INFINITE]
    pub fn process<F: Handler>(&mut self, handler: &F, timeout: f32) -> Pod {
        match self.recv(timeout) {
            Event::Open(id, sender, address) => {
                let ws = WebSocket {
                    id,
                    sender,
                    address,
                };
                self.map.insert(id, ws);
                let ws = &self.map[&id];
                handler.on_open(ws)?;
            }
            Event::Close(id) => {
                let ws = &self.map[&id];
                handler.on_close(ws)?;
            }
            Event::Text(id, s) => {
                let ws = &self.map[&id];
                handler.on_message(ws, s)?;
            }
            Event::Binary(id, buf) => {
                let ws = &self.map[&id];
                handler.on_binary(ws, buf)?;
            }
            Event::Timeout => {
                handler.on_timeout()?;
            }
            Event::Error(error) => {
                handler.on_error(error)?;
            }
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        self.radio.shutdown().ok();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.radio.shutdown().ok();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

/// Listen on endpoint, such as "127.0.0.1:3012"
pub fn listen(address: &str) -> anyhow::Result<Server, anyhow::Error> {
    let (tx, rx) = channel();
    let id = Arc::new(AtomicU32::new(0));
    let n = id.clone();

    let socket = ws::Builder::new()
        .build(move |out: ws::Sender| {
            let next = n.fetch_add(1, Relaxed);
            WebSocketHandler {
                id: next,
                ws: out,
                tx: tx.clone(),
            }
        })
        .unwrap();

    let address = address.to_string();
    let radio = socket.broadcaster();
    let thread = thread::spawn(move || {
        debug!("listen on: {address}");
        socket.listen(address)?;
        debug!("listen exit");
        Pod::Ok(())
    });

    let t;

    // finished too early
    if thread.is_finished() {
        if let Ok(ret) = thread.join() {
            ret?
        }
        t = None;
    } else {
        t = Some(thread);
    }

    let server = Server {
        rx,
        radio,
        thread: t,
        map: HashMap::new(),
    };

    Ok(server)
}

struct WebSocketHandler {
    id: u32,
    ws: Sender,
    tx: std::sync::mpsc::Sender<Event>,
}

impl ws::Handler for WebSocketHandler {
    fn on_open(&mut self, remote: Handshake) -> Result<()> {
        let mut address = "0.0.0.0".to_string();
        if let Some(s) = &remote.peer_addr {
            address = s.to_string();
        }
        let _ = self.tx.send(Event::Open(self.id, self.ws.clone(), address));
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(x) => {
                let _ = self.tx.send(Event::Text(self.id, x));
            }
            Message::Binary(x) => {
                let _ = self.tx.send(Event::Binary(self.id, x));
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        let _ = self.tx.send(Event::Close(self.id));
    }

    fn on_error(&mut self, err: Error) {
        let _ = self.tx.send(Event::Error(err));
    }
}
