/* Listen for keyboard, mouse and trackpad events and send them to the model.
 *
 */

use clap::Parser;
use fun::{Args, internal_main};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    internal_main(args).await;
}
