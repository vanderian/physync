use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::from_utf8;

use log::{debug, error};
use rand::random;

use crate::packet::{PacketReader, PacketType};
use crate::{errors::Result, Socket};

pub struct Server {
    socket: Socket,
    clients: HashMap<SocketAddr, Client>,
    pending: HashMap<SocketAddr, Client>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self> {
        let socket = Socket::bind(addr).await?;
        println!("Listening on: {}", socket.local_addr()?);

        Ok(Server {
            socket,
            clients: HashMap::new(),
            pending: HashMap::new(),
        })
    }

    pub fn process_packet(
        addr: SocketAddr,
        data: &[u8],
        clients: &mut HashMap<SocketAddr, Client>,
        pending: &mut HashMap<SocketAddr, Client>,
    ) -> Result<()> {
        let mut reader = PacketReader::new(data);
        let header = reader.read_base_header()?;
        if !header.is_current_protocol() {
            debug!("protocol version does not match");
            //    send error
        }

        match header.packet_type() {
            PacketType::Connect => {
                let session = reader.read_session_header()?;
                match (clients.get(&addr), pending.get(&addr)) {
                    (None, None) => {
                        let server_id = random::<u64>();
                        pending.insert(addr, Client::new(addr, session.session_id(), server_id));
                        debug!(
                            "connection request from: {} {} send server id:{}",
                            addr,
                            session.session_id(),
                            server_id
                        );
                        // send server_id
                    }
                    (None, Some(c)) => {
                        if c.challenge() == session.session_id() {
                            clients.insert(addr, pending.remove(&addr).unwrap());
                            debug!("connected from: {}", addr);
                        //    send connected confirmation ? should be enough to start sending data packets
                        } else if c.client_id != session.session_id() {
                            debug!("challenge error from: {}", addr);
                            //    send error
                        }
                        debug!("ignore con request from: {}", addr);
                        // ignore connection request packets with client id
                    }
                    _ => debug!("ignore all from: {}", addr), // ignore the rest
                }
            }
            PacketType::Data => {
                let session = reader.read_session_header()?;
                match clients.get(&addr) {
                    Some(c) if c.challenge() == session.session_id() => {
                        let payload = reader.read_payload();
                        println!("{:?}: {:?}", c, from_utf8(&payload).unwrap())
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub async fn read_loop(self) -> Result<()> {
        let Server {
            mut socket,
            mut clients,
            mut pending,
        } = self;
        loop {
            let (addr, data) = socket.recv().await?;
            match Server::process_packet(addr, data, &mut clients, &mut pending) {
                Err(e) => {
                    error!("Encountered an error receiving data: {:}", e);
                    continue;
                }
                _ => (),
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Client {
    addr: SocketAddr,
    client_id: u64,
    server_id: u64,
}

impl Client {
    pub fn new(addr: SocketAddr, client_id: u64, server_id: u64) -> Self {
        Client {
            addr,
            client_id,
            server_id,
        }
    }

    pub fn challenge(&self) -> u64 {
        self.client_id ^ self.server_id
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
