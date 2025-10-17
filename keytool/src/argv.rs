use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "keytool", about = "HodeauxLedger Key Tool")]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Sign(SignArgs),
    Verify(VerifyArgs),
    Generate(GenerateArgs),
    View(ViewArgs),
    Base64(B64Args),
    SelfID(SelfIDArgs),
}

#[derive(Args, Debug)]
pub struct KeyOpts {
    #[arg(short, long)]
    pub keyfile: String,
    #[arg(short, long)]
    pub password: Option<String>,
    #[arg(long)]
    pub hot: bool,
}

#[derive(Args, Debug)]
pub struct SignArgs {
    #[command(flatten)]
    pub key: KeyOpts,

    #[arg(short, long)]
    pub sig_type: String,

    #[arg(short, long)]
    pub input: String,

    #[arg(short, long, value_name = "DIR")]
    pub output: String,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[arg(short, long)]
    pub input: String,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    #[command(flatten)]
    pub key: KeyOpts,

    #[arg(long)]
    pub show_sk: bool,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    #[command(flatten)]
    pub key: KeyOpts,

    #[arg(long)]
    pub show_sk: bool,
}

#[derive(Args, Debug)]
pub struct B64Args {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub output: String,
}

#[derive(Args, Debug)]
pub struct SelfIDArgs {
    #[arg(short, long)]
    pub keyfile: String,
}
