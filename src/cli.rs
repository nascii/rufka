#![allow(needless_pass_by_value)]

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

    #[structopt(name = "create")]
    Create(CreateCommand),

    #[structopt(name = "publish")]
    Publish(PublishCommand),

    #[structopt(name = "subscribe")]
    Subscribe(SubscribeCommand),
}

#[derive(StructOpt, Debug)]
struct StartCommand {}

#[derive(StructOpt, Debug)]
struct StatusCommand {}

#[derive(StructOpt, Debug)]
struct CreateCommand {
    topic: String,
}

#[derive(StructOpt, Debug)]
struct PublishCommand {
    topic: String,
    #[structopt(default_value = "")]
    key: String,
}

#[derive(StructOpt, Debug)]
struct SubscribeCommand {
    topic: String,
}

fn start(_common: Common, _command: StartCommand) {
    server::start();
}

fn status(_common: Common, _command: StatusCommand) {
    client::status();
}

fn create(_common: Common, command: CreateCommand) {
    client::create(command.topic);
}

fn publish(_common: Common, command: PublishCommand) {
    client::publish(command.topic, command.key);
}

fn subscribe(_common: Common, command: SubscribeCommand) {
    client::subscribe(command.topic);
}

pub fn init() {
    let CLI { common, command } = CLI::from_args();

    match command {
        Command::Start(cmd) => start(common, cmd),
        Command::Status(cmd) => status(common, cmd),
        Command::Create(cmd) => create(common, cmd),
        Command::Publish(cmd) => publish(common, cmd),
        Command::Subscribe(cmd) => subscribe(common, cmd),
    }
}
