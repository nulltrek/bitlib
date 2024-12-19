use std::fmt;
pub enum Compression {
    Yes,
    No,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Network {
    Test,
    Main,
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Network::Main => "mainnet",
            Network::Test => "testnet",
        };

        write!(f, "{}", name)
    }
}
