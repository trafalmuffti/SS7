mod app;
mod ssh;
mod ui;

use std::sync::Arc;

use ss7_billing::BillingDb;

#[tokio::main]
async fn main() {
    env_logger::init();

    let db = BillingDb::open("ss7_billing.db").expect("Failed to open database");
    let db = Arc::new(db);

    let addr = "0.0.0.0:2222";
    println!("SS7 Billing TUI SSH Server");
    println!("Listening on {}", addr);
    println!("Connect with: ssh root@<host> -p 2222");

    ssh::run_ssh_server(addr, db).await.expect("SSH server failed");
}
