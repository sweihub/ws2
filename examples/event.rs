use log2::*;
use ws2::Pod;

fn main() -> Pod {
    let _log2 = log2::start();
    let mut ws = ws2::connect("ws://172.16.210.211:5600");

    loop {
        let event = ws.recv(0.01);
        match event {
            ws2::client::Event::Open(_) => {
                info!("open");
                if let Err(e) = ws.send("hello world") {
                    error!("{e}");
                }
            }
            ws2::client::Event::Close => {
                info!("close");
            }
            ws2::client::Event::Text(msg) => {
                info!("{msg}");
            }
            ws2::client::Event::Binary(_) => {}
            ws2::client::Event::Timeout => {}
            ws2::client::Event::Error(error) => {
                error!("{error}");
            }
        }
    }
}
