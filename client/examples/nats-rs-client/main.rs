use client;
use quicli::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    /// NAT server, provider default demo server
    #[structopt(long, short, default_value = "nats://demo.nats.io")]
    server: String,

    /// Command: pub, sub, request, reply
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    /// The type of operation, can be one of pub, sub, qsub, req, reply.
    #[structopt(name = "pub", about = "Publishes a message to a given subject")]
    Pub { subject: String, msg: String },
    #[structopt(name = "sub", about = "Subscribes to a given subject")]
    Sub { subject: String },
    #[structopt(name = "request", about = "Sends a request and waits on reply")]
    Request { subject: String, msg: String },
    #[structopt(name = "reply", about = "Listens for requests and sends the reply")]
    Reply { subject: String, resp: String },
}

fn main() -> CliResult {
    let args = Cli::from_args();
    let mut nc = client::Client::new(args.server).unwrap();

    match args.cmd {
        Command::Pub { subject, msg } => {
            unimplemented!() // TODO
        }
        Command::Sub { subject } => {
            let sub = nc.subscribe(&subject, None).unwrap();
            println!("Listening on {}", subject);
        }
        _ => {
            unimplemented!() // TODO
        }
    }

    Ok(())
}
