use crate::errors::Result;
use crate::{Peer, Packet};
use std::net::SocketAddr;
use std::time::Instant;

pub struct Server {
    peer: Peer,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self> {
        let peer = Peer::bind(addr).await?;
        println!("Listening on: {}", peer.local_addr()?);

        Ok(Server { peer })
    }

    pub async fn run(self) -> Result<()> {
        let mut peer = self.peer;
        peer.in_loop().await;
        Ok(())
    }
}

pub struct Client {
    peer: Peer,
    remote: SocketAddr,
}

impl Client {
    pub async fn new(addr: &str) -> Result<Self> {
        let peer = Peer::bind_any().await?;
        println!("Listening on: {}", peer.local_addr()?);

        Ok(Client {
            peer,
            remote: addr.parse().unwrap(),
        })
    }

    pub async fn connect(&mut self) {
        self.peer.connect(self.remote).await.unwrap();
    }

    pub async fn poll(&mut self) {
        self.peer.manual_poll(Instant::now()).await.unwrap();
    }

    pub async fn loop_send(&mut self) {
        let p = Packet::new(self.remote, Box::from("hello".as_bytes()));
        self.peer.loop_send(&p).await.unwrap();
    }
}

/*
pub async fn run(self) -> Result<(), io::Error> {
    let Server {
        mut socket,
        mut buf,
    } = self;

    let mut to_send: Option<(usize, SocketAddr)> = None;
    let mut receivers: BTreeSet<String> = BTreeSet::new();
    let mut owner: String = "".to_string();

    loop {
        // First we check to see if there's a message we need to relay back.
        // If so then we try to relay it to the receivers, waiting
        // until it's writable and we're able to do so.
        match to_send {
            Some((size, peer)) if peer.to_string() == owner => {
                let msg = from_utf8(&buf[..size]).unwrap();

                for rec in &receivers {
                    socket.send_to(&buf[..size], rec).await?;
                    println!("Relayed ({}) to {}", msg, rec);
                }
            }
            _ => ()
        }

        // If we're here then `to_send` is `None`, so we take a look for the
        // next message we're going to relay back.
        to_send = Some(socket.recv_from(&mut buf).await?);
        let sender = to_send.unwrap().1.to_string();
        if owner.is_empty() {
            println!("Got owner: {}", sender);
            owner = sender;
        } else if !receivers.contains(&sender) && sender != owner {
            println!("Got recipient: {}", sender);
            receivers.insert(sender);
        }
    }
*/
