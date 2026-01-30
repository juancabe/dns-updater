use std::env;

use dns_updater::{dyn_dns::parse_dns_tuples, runner::Runner};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let iface = env::var("INTERFACE").expect("The INTERFACE env flag should be set");
    let dyn_dnss =
        parse_dns_tuples(&env::var("DNS_TUPLES").expect("You should supply some DNS_TUPLES"))
            .unwrap();

    let runner = Runner::new(iface, dyn_dnss).unwrap();

    runner.run().await
}
