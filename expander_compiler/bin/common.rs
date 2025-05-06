use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ExpanderExecArgs {
    #[arg(short, long, default_value = "M31")]
    pub field_type: String,
}
