use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "usher", about = "R⬢ Publishing Tool - Usher")]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Submit(SubmitArgs),
    Request(RequestArgs),
    Mirror(MirrorArgs),
    Find(FindArgs),
    Head(HeadArgs),
}

#[derive(Args, Debug)]
pub struct SubmitArgs {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long)]
    pub host: Option<String>,

    #[arg(long)]
    pub port: Option<String>,

    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct RequestArgs {
    #[arg(long)]
    pub host: String,

    #[arg(short, long)]
    pub port: String,

    #[arg(short, long)]
    pub scope: String,

    #[arg(short, long)]
    pub keyfile: String,
}

#[derive(Args, Debug)]
pub struct MirrorArgs {
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long)]
    pub host: Option<String>,

    #[arg(long)]
    pub port: Option<String>,

    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct FindArgs {
    #[arg(short, long)]
    pub scope: String,

    #[arg(long)]
    pub host: String,

    #[arg(short, long)]
    pub port: String,
}

#[derive(Args, Debug)]
pub struct HeadArgs {
    #[arg(short, long)]
    pub scope: String,

    #[arg(short, long)]
    pub keyfile: String,

    #[arg(short, long)]
    pub cache_dir: String,
}
