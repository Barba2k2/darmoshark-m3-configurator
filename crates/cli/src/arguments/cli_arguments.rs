use clap::Parser;

use crate::arguments::command::Command;

/// Configure a Darmoshark M3 (Attack Shark M3) mouse.
#[derive(Debug, Parser)]
#[command(
  name = "dms",
  after_help = "Writes work over the cable (feature report 0x52) and over the 2.4GHz \
                receiver (feature report 0x51). Reads marked [receiver] need the \
                receiver: the cable always answers with the identity block instead \
                of the stored configuration."
)]
pub struct CliArguments {
  #[command(subcommand)]
  pub command: Command,
}
