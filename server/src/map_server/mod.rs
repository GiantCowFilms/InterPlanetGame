use h2::server;
use http::{Response, StatusCode};
use tokio::net::TcpListener;
use tokio::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use crate::game_server::map_manager;

pub struct MapServer {

}

impl MapServer {
    pub fn start(port: u16, maps: impl map_manager::MapManager + Send + 'static) {
        env_logger::init();
        let listener = TcpListener::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        )).unwrap();
        tokio::run_async(async move {
            let mut incoming = listener.incoming();
            while let Some(stream) = await!(incoming.next()) {
                let mut h2 = await!(server::handshake(stream.expect("1"))).expect("2");
                while let Some(Ok((request, mut respond))) = await!(h2.next()) {
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .body(())
                        .unwrap();

                    respond.send_response(response, true).unwrap(); //Does this block??
                }
            }
        });
    }
}
