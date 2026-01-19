// TODO glacier should register Extension #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

use crate::core::IglooRequest;

mod core;
mod ext;
mod query;
mod tree;
mod web;

#[tokio::main]
async fn main() {
    let (handle, req_tx) = core::spawn().await.unwrap();

    if let Err(e) = web::run(req_tx.clone()).await {
        eprintln!("Error running web: {e}");
    }

    tokio::signal::ctrl_c().await.unwrap();
    println!("SHUTTING DOWN");
    req_tx.send(IglooRequest::Shutdown).unwrap();
    handle.join().unwrap();
}
