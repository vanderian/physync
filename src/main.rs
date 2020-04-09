use std::error::Error;
use std::result;

use clap::{App, AppSettings, ArgMatches, load_yaml};
use env_logger::Builder;
use futures::TryFutureExt;
use log::LevelFilter;

use physync::client::Client;
use physync::server::Server;

#[tokio::main]
async fn main() -> result::Result<(), Box<dyn Error>> {
    Builder::default().filter_level(LevelFilter::Trace).init();

    let yaml = load_yaml!("cli/cli.yml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    if let Some(m) = matches.subcommand_matches("server") {
        run_server(m.to_owned()).await?;
    }
    if let Some(m) = matches.subcommand_matches("client") {
        run_client(m.to_owned()).await?;
    }

    Ok(())
}

async fn run_server(m: ArgMatches<'_>) -> result::Result<(), Box<dyn Error>> {
    let host = m.value_of("LISTEN_HOST").unwrap();
    Server::new(host).and_then(Server::run).await?;

    Ok(())
}

async fn run_client(m: ArgMatches<'_>) -> result::Result<(), Box<dyn Error>> {
    let host = m.value_of("CONNECT_ADDR").unwrap();
    Client::new(host).and_then(Client::run).await?;

    Ok(())
}
