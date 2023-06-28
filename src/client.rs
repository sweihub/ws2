use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{sync::mpsc::RecvTimeoutError, thread};
use ws::{CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

use crate::{Pod, WebSocket, INFINITE};

pub struct Client {
    thread: Option<JoinHandle<()>>,
    radio: Option<Sender>,
    sender: Option<Sender>,
    rx: Option<EventReceiver>,
    input: std::sync::mpsc::Receiver<(Sender, EventReceiver)>,
    run: std::sync::mpsc::Sender<bool>,
    address: String,
    ws: Option<WebSocket>,
}

type EventReceiver = std::sync::mpsc::Receiver<Event>;
type EventSender = std::sync::mpsc::Sender<Event>;

pub enum Event {
    Open(Sender),
    Close,
    Text(String),
    Binary(Vec<u8>),
    Timeout,
    Error(Error),
}

impl Client {
    /// Receive the events, timeout is decimal seconds
    pub fn recv(&mut self, timeout: f32) -> Event {
        let mut n = Duration::from_nanos(std::u64::MAX);
        if timeout != INFINITE {
            n = Duration::from_nanos((timeout * 1e9) as u64)
        }

        if self.rx.is_none() {
            if let Ok((radio, rx)) = self.input.recv_timeout(n) {
                self.radio = Some(radio);
                self.rx = Some(rx);
            }
        } else if let Ok((radio, rx)) = self.input.try_recv() {
            self.radio = Some(radio);
            self.rx = Some(rx);
        }

        if let Some(rx) = &self.rx {
            return match rx.recv_timeout(n) {
                Ok(event) => event,
                Err(e) => match e {
                    RecvTimeoutError::Timeout => Event::Timeout,
                    RecvTimeoutError::Disconnected => {
                        self.radio = None;
                        self.rx = None;
                        self.sender = None;
                        return Event::Error(Error::new(ErrorKind::Internal, "disconnected"));
                    }
                },
            };
        } else {
            // wait for connection
            std::thread::sleep(n);
            return Event::Error(Error::new(ErrorKind::Internal, "not connected"));
        }
    }

    /// Shutdown the socket
    pub fn shutdown(&self) {
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
pub fn connect(url: &str) -> anyhow::Result<Client> {
    // mover
    let (ms, mr) = channel();
    // flag
    let (fs, fr) = channel();

    let address = url.to_string();
    let url = url::Url::parse(&address)?;

    let thread = thread::spawn(move || {
        let mut run = true;
        while run {
            let (tx, rx) = channel();
            let mut socket = ws::Builder::new()
                .build(move |sender| WebSocketHandler {
                    ws: sender,
                    tx: tx.clone(),
                })
                .unwrap();

            // send to parent thread
            let radio = socket.broadcaster();
            let _ = ms.send((radio, rx));

            let to = url.clone();
            let _ = socket.connect(to);
            let _ = socket.run();

            if let Ok(x) = fr.recv_timeout(Duration::from_secs(3)) {
                run = x;
            }
        }
    });

    let c = Client {
        thread: Some(thread),
        radio: None,
        rx: None,
        input: mr,
        run: fs,
        sender: None,
        ws: None,
        address,
    };

    Ok(c)
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
