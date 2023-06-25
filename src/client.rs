use crossbeam::channel::unbounded as channel;
use crossbeam::channel::RecvTimeoutError;
use log2::*;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use ws::{CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

pub const INFINITE: f32 = std::f32::MAX;

pub struct Connection {
    thread: Option<JoinHandle<()>>,
    radio: Option<Sender>,
    rx: Option<EventReceiver>,
    xr: std::sync::mpsc::Receiver<(Option<Sender>, Option<EventReceiver>)>,
}

type EventReceiver = crossbeam::channel::Receiver<Event>;
type EventSender = crossbeam::channel::Sender<Event>;

impl Connection {
    /// timeout with decimal seconds
    pub fn recv(&mut self, timeout: f32) -> Event {
        let n = if timeout == INFINITE {
            Duration::from_nanos(std::u64::MAX)
        } else {
            Duration::from_nanos((timeout * 1e9) as u64)
        };

        if let Ok((radio, rx)) = self.xr.try_recv() {
            self.radio = radio;
            self.rx = rx;
        }

        // wait for connection
        if self.rx.is_none() {
            std::thread::sleep(n);
            return Event::Error(Error::new(ErrorKind::Internal, "not connected"));
        }

        let rx = self.rx.as_ref().unwrap();
        return match rx.recv_timeout(n) {
            Ok(event) => event,
            Err(e) => match e {
                RecvTimeoutError::Timeout => Event::Timeout,
                RecvTimeoutError::Disconnected => {
                    Event::Error(Error::new(ErrorKind::Internal, e.to_string()))
                }
            },
        };
    }

    #[inline]
    pub fn send<M>(&self, msg: M) -> Result<()>
    where
        M: Into<Message>,
    {
        if let Some(radio) = &self.radio {
            return radio.send(msg);
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
        //self.radio.shutdown().ok();
        let thread = self.thread.take();
        thread.unwrap().join().ok();
    }
}

pub fn connect(url: &str) -> anyhow::Result<Connection> {
    let (xs, xr) = std::sync::mpsc::channel();

    let url = url.to_string();
    let thread = thread::spawn(move || loop {
        let (tx, rx) = channel();

        let mut socket = ws::Builder::new()
            .build(move |sender| Client {
                ws: sender,
                tx: tx.clone(),
            })
            .unwrap();

        let radio = socket.broadcaster();
        let _ = xs.send((Some(radio), Some(rx)));

        let to = url.parse().unwrap();
        debug!("connect {to}");
        let _ = socket.connect(to);
        let _ = socket.run();

        debug!("connect exit");
        std::thread::sleep(Duration::from_secs(3));
    });

    let c = Connection {
        thread: Some(thread),
        radio: None,
        rx: None,
        xr,
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

pub enum Event {
    Open(Sender),
    Close,
    Text(String),
    Binary(Vec<u8>),
    Timeout,
    Error(Error),
}
