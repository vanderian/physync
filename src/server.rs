use crate::errors::Result;
use crate::Peer;

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
