use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wgatools")]
#[command(
    about = "a cross-platform and ultrafast toolkit for Whole Genome Alignment Files manipulation"
)]
#[command(long_about = "long_about todo!!!")]
#[command(author, version)]
#[command(
help_template =
"{name} -- {about}\n\nVersion: {version}\n\nAuthors: {author}\
    \n\n{usage-heading} {usage}\n\n{all-args}"
) // change template more!
]
pub struct Cli {
    /// Output file ("-" for stdout)
    #[arg(long, short, global = true, default_value = "-", help_heading = Some("GLOBAL"))]
    pub outfile: String,
    /// Bool, if rewrite output file [default: false]
    #[arg(long, short, global = true, default_value = "false", help_heading = Some("GLOBAL"))]
    pub rewrite: bool,
    // Threads, default 1
    #[arg(long, short, global = true, default_value = "1", help_heading = Some("GLOBAL"))]
    pub threads: usize,
    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert MAF format to PAF format
    #[command(visible_alias = "m2p", name = "maf2paf")]
    Maf2Paf {
        /// Input MAF File, None for STDIN
        #[arg(required = false)]
        input: Option<String>,
    },
    /// Convert MAF format to Chain format
    #[command(visible_alias = "m2c", name = "maf2chain")]
    Maf2Chain {
        /// Input MAF File, None for STDIN
        #[arg(required = false)]
        input: Option<String>,
    },
    /// Convert PAF format to MAF format
    #[command(visible_alias = "p2m", name = "paf2maf")]
    Paf2Maf {
        /// Input PAF File, None for STDIN
        #[arg(required = false)]
        input: Option<String>,
        /// Input target FASTA File, required
        #[arg(required = true, long, short)]
        target: String,
        /// Input query FASTA File, required
        #[arg(required = true, long, short)]
        query: String,
    },
    /// Convert PAF format to Chain format
    #[command(visible_alias = "p2c", name = "paf2chain")]
    Paf2Chain {
        /// Input PAF File, None for STDIN
        #[arg(required = false)]
        input: Option<String>,
    },
    /// Convert Chain format to MAF format
    #[command(visible_alias = "c2m", name = "chain2maf")]
    Chain2Maf {
        /// Input Chain File, None for STDIN
        #[arg(required = false)]
        input: Option<String>,
        /// Input target FASTA File, required
        #[arg(required = true, long, short)]
        target: String,
        /// Input query FASTA File, required
        #[arg(required = true, long, short)]
        query: String,
    },
    /// Convert Chain format to PAF format
    #[command(visible_alias = "c2p", name = "chain2paf")]
    Chain2Paf {
        /// Input Chain File, None for STDIN
        #[arg(required = false)]
        input: Option<String>,
    },
}

pub fn make_cli_parse() -> Cli {
    Cli::parse()
}
