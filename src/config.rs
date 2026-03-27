use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Coral: ZK proofs that a public document matches a committed grammar", long_about = None)]
#[clap(group(
            ArgGroup::new("mode")
                .required(true)
                .args(&["commit", "prove", "verify", "e2e"]),
        ))]
pub struct Options {
    #[arg(long, default_value_t = false, help = "Commit the grammar")]
    pub commit: bool,
    #[arg(long, default_value_t = false, help = "Prove public document against committed grammar")]
    pub prove: bool,
    #[arg(long, default_value_t = false, help = "Verify proof against public document and grammar commitment")]
    pub verify: bool,
    #[arg(long, default_value_t = false, help = "End-to-end: commit, prove, and verify")]
    pub e2e: bool,
    #[arg(long, value_name = "FILE", help = "Optional name for grammar commitment file")]
    pub cmt_name: Option<String>,
    #[arg(long, value_name = "FILE", help = "Optional name for .proof file")]
    pub proof_name: Option<String>,
    #[arg(short = 'd', long, value_name = "FILE", help = "Public document file (required for prove/verify/e2e)")]
    pub doc: Option<String>,
    #[arg(short = 'g', long, value_name = "FILE", help = "Grammar .pest file (required for commit/prove/e2e)")]
    pub grammar: Option<String>,
    #[arg(
        short = 'm',
        long,
        value_name = "FILE",
        help = "Metrics and other output information"
    )]
    pub metrics: Option<PathBuf>,
    #[arg(
        short = 'b',
        long = "batch-size",
        value_name = "USIZE",
        help = "Batch size (override auto select)",
        default_value_t = 1, // auto select
    )]
    pub batch_size: usize,
}
