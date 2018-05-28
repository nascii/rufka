use std::path::PathBuf;

use structopt::StructOpt;

use client;
use server;

#[derive(StructOpt, Debug)]
#[structopt(name = "rufka")]
struct CLI {
    #[structopt(flatten)]
    common: Common,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
struct Common {
    #[structopt(long = "config", short = "c", help = "path to the config", parse(from_os_str))]
    config: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "start")]
    Start(StartCommand),

    #[structopt(name = "status")]
    Status(StatusCommand),
}

#[derive(StructOpt, Debug)]
struct StartCommand {}

#[derive(StructOpt, Debug)]
struct StatusCommand {}

fn start(common: Common, command: StartCommand) {
    server::start();
}

fn status(common: Common, command: StatusCommand) {
    client::status();
}

pub fn init() {
    let CLI { common, command } = CLI::from_args();

    match command {
        Command::Start(cmd) => start(common, cmd),
        Command::Status(cmd) => status(common, cmd),
    }
}
