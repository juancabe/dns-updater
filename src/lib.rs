use if_addrs::get_if_addrs;

pub mod persistence;

#[derive(Default)]
pub struct Runner;

impl Runner {
    pub fn run(&self) {
        let addresses = get_if_addrs().unwrap();
        println!("Hello World, got some addresses: {:?}", addresses);
    }
}

#[cfg(test)]
mod tests {}
