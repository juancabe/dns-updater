use std::{env, path::PathBuf};

use dns_updater::Runner;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    let runner = Runner::new(
        env::var("INTERFACE").expect("The INTERFACE env flag should be set"),
        env::var("POLL_SECS")
            .expect("The POLL_SECS env flag should be set")
            .parse()
            .expect("POLL_SECS should be valid u64"),
        env::var("DATABASE_FILE").ok().map(PathBuf::from).as_ref(),
    );
    runner.run().await
}

