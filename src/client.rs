use log2::*;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{sync::mpsc::RecvTimeoutError, thread};
use ws::{CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

pub const INFINITE: f32 = std::f32::MAX;

pub struct Connection {
    thread: Option<JoinHandle<()>>,
    radio: Option<Sender>,
    ws: Option<Sender>,
    rx: Option<EventReceiver>,
    input: std::sync::mpsc::Receiver<(Sender, EventReceiver)>,
    run: std::sync::mpsc::Sender<bool>,
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

impl Connection {
    /// timeout with decimal seconds
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
                Ok(event) => self.find_websocket(event),
                Err(e) => match e {
                    RecvTimeoutError::Timeout => Event::Timeout,
                    RecvTimeoutError::Disconnected => {
                        self.radio = None;
                        self.rx = None;
                        self.ws = None;
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

    fn find_websocket(&mut self, event: Event) -> Event {
        if let Event::Open(ws) = &event {
            self.ws = Some(ws.clone());
        }
        event
    }

    #[inline]
    pub fn send<M>(&self, msg: M) -> Result<()>
    where
        M: Into<Message>,
    {
        if let Some(ws) = &self.ws {
            return ws.send(msg);
        }
        Err(Error::new(ErrorKind::Internal, "not connected"))
    }

    pub fn close(&self) {
        if let Some(radio) = &self.radio {
            radio.shutdown().ok();
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(ws) = &self.ws {
            ws.close(CloseCode::Normal).ok();
        }
        if let Some(radio) = &self.radio {
            radio.shutdown().ok();
        }
        self.run.send(false).ok();
        let thread = self.thread.take();
        thread.unwrap().join().ok();
    }
}

pub fn connect(url: &str) -> anyhow::Result<Connection> {
    // mover
    let (ms, mr) = channel();
    // flag
    let (fs, fr) = channel();

    let url = url::Url::parse(url)?;

    let thread = thread::spawn(move || {
        let mut run = true;
        while run {
            let (tx, rx) = channel();
            let mut socket = ws::Builder::new()
                .build(move |sender| Client {
                    ws: sender,
                    tx: tx.clone(),
                })
                .unwrap();

            // send to parent thread
            let radio = socket.broadcaster();
            let _ = ms.send((radio, rx));

            debug!("connect {url}");
            let to = url.clone();
            let _ = socket.connect(to);
            let _ = socket.run();

            debug!("connect exit");
            if let Ok(x) = fr.recv_timeout(Duration::from_secs(3)) {
                run = x;
            }
        }
    });

    let c = Connection {
        thread: Some(thread),
        radio: None,
        rx: None,
        input: mr,
        run: fs,
        ws: None,
    };

    Ok(c)
}

struct Client {
    ws: Sender,
    tx: EventSender,
}

impl Handler for Client {
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
