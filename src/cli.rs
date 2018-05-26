use std::path::PathBuf;

use structopt::StructOpt;
use tokio;

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

    #[structopt(name = "start")]
    Stop(StopCommand),

    #[structopt(name = "status")]
    Status(StatusCommand),
}

#[derive(StructOpt, Debug)]
struct StartCommand {}

#[derive(StructOpt, Debug)]
struct StopCommand {}

#[derive(StructOpt, Debug)]
struct StatusCommand {}

fn start(common: Common, command: StartCommand) {
    let srv = server::create();

    tokio::run(srv);
}

fn stop(common: Common, command: StopCommand) {}

fn status(common: Common, command: StatusCommand) {}

pub fn init() {
    let CLI { common, command } = CLI::from_args();

    match command {
        Command::Start(cmd) => start(common, cmd),
        Command::Stop(cmd) => stop(common, cmd),
        Command::Status(cmd) => status(common, cmd),
    }
}
