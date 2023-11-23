use crate::{Pod, WebSocket};
use log2::info;
use poll_channel::*;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use ws::{CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

pub struct Client {
    thread: Option<JoinHandle<()>>,
    radio: Option<Sender>,
    sender: Option<Sender>,
    ws: Option<WebSocket>,
    rx: EventReceiver,
    run: std::sync::mpsc::Sender<bool>,
    connector: std::sync::mpsc::Receiver<Sender>,
    address: String,
}

type EventReceiver = poll_channel::Receiver<Event>;
type EventSender = poll_channel::Sender<Event>;

pub enum Event {
    Open(Sender),
    Close,
    Text(String),
    Binary(Vec<u8>),
    Timeout,
    Error(Error),
}

impl Client {
    /// Create a empty client
    pub fn new() -> Self {
        let (_tx, rx) = poll_channel::channel();
        let (run, _exit) = std::sync::mpsc::channel();
        let (_mover, connector) = std::sync::mpsc::channel();
        Self {
            thread: None,
            radio: None,
            sender: None,
            ws: None,
            rx,
            run,
            connector,
            address: String::new(),
        }
    }

    pub fn connect(&mut self, url: &str) -> Pod {
        let _ = url::Url::parse(url)?;
        *self = connect(url);
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.sender.is_some()
    }

    /// Receive the events, timeout is decimal seconds
    pub fn recv(&mut self, timeout: f32) -> Event {
        let n = Duration::from_nanos((timeout as f64 * 1e9) as u64);

        // reconnect?
        if let Ok(radio) = self.connector.try_recv() {
            self.radio = Some(radio);
        }

        match self.rx.recv_timeout(n) {
            Ok(event) => {
                if let Event::Open(sender) = &event {
                    self.sender = Some(sender.clone());
                }
                return event;
            }
            Err(error) => match error {
                RecvTimeoutError::Timeout => return Event::Timeout,
                RecvTimeoutError::Disconnected => {
                    self.radio = None;
                    self.sender = None;
                    return Event::Error(Error::new(ErrorKind::Internal, error.to_string()));
                }
            },
        };
    }

    pub fn send<M>(&self, msg: M) -> Pod
    where
        M: Into<ws::Message>,
    {
        if let Some(sender) = &self.sender {
            Ok(sender.send(msg)?)
        } else {
            Err(anyhow::anyhow!("websocket not connected"))
        }
    }

    /// Close the socket
    pub fn close(&self) {
        if let Some(radio) = &self.radio {
            radio.shutdown().ok();
        }
    }

    /// Process events with decimal seconds timeout, or [INFINITE]
    pub fn process<F: crate::Handler>(&mut self, handler: &mut F, timeout: f32) -> Pod {
        loop {
            match self.recv(timeout) {
                Event::Open(sender) => {
                    let ws = WebSocket {
                        id: 0,
                        sender,
                        address: self.address.clone(),
                    };
                    self.ws = Some(ws);
                    handler.on_open(self.ws.as_mut().unwrap())?;
                }
                Event::Close => {
                    handler.on_close(self.ws.as_mut().unwrap())?;
                }
                Event::Text(msg) => {
                    handler.on_message(self.ws.as_mut().unwrap(), msg)?;
                }
                Event::Binary(buf) => {
                    handler.on_binary(self.ws.as_mut().unwrap(), buf)?;
                }
                Event::Timeout => {
                    handler.on_timeout()?;
                    break;
                }
                Event::Error(error) => {
                    handler.on_error(error)?;
                    break;
                }
            }
        }
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(radio) = &self.radio {
            radio.shutdown().ok();
        }
        self.run.send(false).ok();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

/// Connect to WebSocket, such as `ws://1.2.3.4:5678/`
pub fn connect(url: &str) -> Client {
    let (tx, rx) = poll_channel::channel();
    let (mover, connector) = channel();
    let (notify, exit) = channel();
    let address = url.to_string();

    let thread = thread::spawn(move || {
        loop {
            let inner = tx.clone();
            let mut socket = ws::Builder::new()
                .build(move |sender| WebSocketHandler {
                    ws: sender,
                    tx: inner.clone(),
                })
                .unwrap();

            // send to parent thread
            let radio = socket.broadcaster();
            let _ = mover.send(radio);

            // run
            let url = url::Url::parse(&address);
            match url {
                Ok(to) => {
                    let _ = socket.connect(to);
                    let _ = socket.run();
                }
                Err(error) => {
                    let e = Error::new(ErrorKind::Internal, error.to_string());
                    let _ = tx.send(Event::Error(e));
                    break;
                }
            };

            // sleep and detect exit
            if let Ok(_) = exit.recv_timeout(Duration::from_secs(3)) {
                break;
            }
        }
    });

    Client {
        thread: Some(thread),
        radio: None,
        sender: None,
        ws: None,
        rx,
        run: notify,
        connector,
        address: url.into(),
    }
}

struct WebSocketHandler {
    ws: Sender,
    tx: EventSender,
}

impl Handler for WebSocketHandler {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        let _ = self.tx.send(Event::Open(self.ws.clone()));
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(x) => {
                let _ = self.tx.send(Event::Text(x));
            }
            Message::Binary(x) => {
                let _ = self.tx.send(Event::Binary(x));
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        let _ = self.tx.send(Event::Close);
    }

    fn on_error(&mut self, err: Error) {
        let _ = self.tx.send(Event::Error(err));
    }
}

impl Pollable for Client {
    fn signal(&self) -> ArcMutex2<OptionSignal> {
        self.rx.signal().clone()
    }

    fn id(&self) -> i32 {
        self.rx.id()
    }
}
