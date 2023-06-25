pub use crate::client::INFINITE;
use log2::debug;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::channel;
use std::sync::mpsc::RecvTimeoutError::{Disconnected, Timeout};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use ws::{CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

type Pod = anyhow::Result<(), anyhow::Error>;

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
}

impl Server {
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
}

impl Drop for Server {
    fn drop(&mut self) {
        self.radio.shutdown().ok();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

/// "127.0.0.1:3012"
pub fn listen(address: &str) -> anyhow::Result<Server, anyhow::Error> {
    let (tx, rx) = channel();
    let id = Arc::new(AtomicU32::new(0));
    let n = id.clone();

    let socket = ws::Builder::new()
        .build(move |out: ws::Sender| {
            let next = n.fetch_add(1, Relaxed);
            WebSocket {
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

    let s = Server {
        rx,
        radio,
        thread: t,
    };

    Ok(s)
}

struct WebSocket {
    id: u32,
    ws: Sender,
    tx: std::sync::mpsc::Sender<Event>,
}

impl Handler for WebSocket {
    fn on_open(&mut self, remote: Handshake) -> Result<()> {
        let address = remote.remote_addr().unwrap_or(None);
        let address = address.unwrap_or("".into());
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
