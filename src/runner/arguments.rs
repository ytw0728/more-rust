
use clap::Parser;

#[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
#[command()]
pub struct Args {
   /// maximum lines of contents window
   #[arg(short, long)]
   pub lines: Option<i32>,

   pub path: String
}
