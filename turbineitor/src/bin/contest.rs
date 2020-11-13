use tokio;

use turbineitor::server;

#[tokio::main]
async fn main() {

    println!("going to serve stuff!");
    server::serve_everything().await;
}
