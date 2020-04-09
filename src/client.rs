use std::net::SocketAddr;
use std::time::{Duration, Instant};

use log::debug;
use rand::{random, Rng, thread_rng};
use tokio::net::udp::{RecvHalf, SendHalf};
use tokio::net::UdpSocket;
use tokio::time::interval_at;

use crate::{OutgoingPacketBuilder, Packet};
use crate::errors::Result;
use crate::net::constants::{CONNECT_PAYLOAD_SIZE, DEFAULT_MTU};
use crate::packet::{PacketReader, PacketType};

pub struct Client {
    socket: UdpSocket,
    remote: SocketAddr,
    id: u64,
    buf: Vec<u8>,
    session: u64,
}

impl Client {
    pub async fn new(addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        println!("Listening on: {}", socket.local_addr()?);

        Ok(Client {
            socket,
            remote: addr.parse().unwrap(),
            id: random(),
            buf: vec![0; DEFAULT_MTU as usize],
            session: 0,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        let mut payload = [0_u8; CONNECT_PAYLOAD_SIZE];
        thread_rng().fill(&mut payload[..]);

        // connection request
        let out = OutgoingPacketBuilder::new(&payload)
            .with_default_header(PacketType::Connect)
            .with_session_header(0)
            .with_session_header(self.id)
            .build();
        let packet = Packet::new(self.remote, out.contents());

        self.socket.send_to(packet.payload(), packet.addr()).await?;

        let (size, _) = self.socket.recv_from(self.buf.as_mut()).await?;
        let mut reader = PacketReader::new(&self.buf[..size]);
        let session = reader.read_session_header()?;
        self.session = session.session_id();

        // challenge response
        let out = OutgoingPacketBuilder::new(&payload)
            .with_default_header(PacketType::Connect)
            .with_session_header(self.session)
            .with_session_header(self.id)
            .build();
        let packet = Packet::new(self.remote, out.contents());
        self.socket.send_to(packet.payload(), packet.addr()).await?;

        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        self.connect().await?;

        let (rx, tx) = self.socket.split();
        tokio::spawn(Client::reading(rx));
        Client::sending(tx, self.session, self.remote).await?;

        Ok(())
    }

    pub async fn reading(mut rx: RecvHalf) -> Result<()> {
        let mut buf: Vec<u8> = vec![0; DEFAULT_MTU as usize];
        loop {
            let time = Instant::now();
            let (size, _) = rx.recv_from(&mut buf).await?;
            let mut reader = PacketReader::new(&buf[..size]);
            let header = reader.read_base_header()?;

            debug!("received {:?} @{:?}", header.packet_type(), time.elapsed())
        }
    }


    pub async fn sending(mut tx: SendHalf, session: u64, remote: SocketAddr) -> Result<()> {
        let mut i = interval_at(
            tokio::time::Instant::now() + Duration::from_secs(3),
            Duration::from_millis(10),
        );
        let mut payload = [0_u8; 128];
        thread_rng().fill(&mut payload);
        loop {
            let time = Instant::now();
            i.tick().await;
            let out = OutgoingPacketBuilder::new(&payload)
                .with_default_header(PacketType::Data)
                .with_session_header(session)
                .build();
            tx.send_to(&out.contents(), &remote).await?;
            debug!("send data @{:?}", time.elapsed());
        }
    }
}
