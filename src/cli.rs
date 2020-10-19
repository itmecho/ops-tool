use crate::tool::{Tool, TOOL_NAMES};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ops-tool",
    about = "CLI tool for managing installed versions of ops tools"
)]
pub struct Cli {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// List the status of currently installed tools
    Status {
        /// Optionally show the status of a single tool
        #[structopt(possible_values = TOOL_NAMES)]
        tool: Option<String>,
    },
    /// Switch to or install the given tool and version
    Use {
        /// Forces the tool to be downloaded even if it already exists locally
        #[structopt(long, short)]
        force: bool,

        /// The name of the tool
        #[structopt(possible_values = TOOL_NAMES)]
        tool: Tool,

        /// The version to switch to or install
        version: String,
    },
}
