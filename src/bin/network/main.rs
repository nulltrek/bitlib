use bitlib::block::BlockId;
use bitlib::definitions::Network;
use bitlib::network::{Envelope, GetHeadersMessage, HeadersMessage, NetAddr, VersionMessage};
use bitlib::u256::U256;
use std::env;
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please specify node ip:port");
        return Err(());
    }

    let version = Envelope::new(
        Network::Main,
        "version",
        VersionMessage {
            version: 70015,
            services: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            addr_recv: NetAddr {
                time: None,
                service: 0,
                ip: [0; 16],
                port: 8333,
            },
            addr_from: NetAddr {
                time: None,
                service: 0,
                ip: [0; 16],
                port: 8333,
            },
            nonce: 10,
            user_agent: "/ua/".to_owned(),
            start_height: 0,
            relay: None,
        }
        .serialize(),
    )
    .unwrap();

    let mut stream = match TcpStream::connect(&args[1]) {
        Err(err) => {
            println!("Error: {}", err.to_string());
            return Err(());
        }
        Ok(stream) => stream,
    };
    println!("* Connection established");

    println!("* Sending version message");
    match stream.write(&version.serialize()) {
        Err(err) => println!("Error: {}", err.to_string()),
        Ok(size) => println!("Wrote {} bytes", size),
    }

    {
        let mut reader = BufReader::new(&mut stream);
        println!("* Receiving version message");
        let response = Envelope::parse(&mut reader).unwrap();
        let version = VersionMessage::parse(&mut response.payload.as_slice()).unwrap();
        println!(
            "Received version {}, with user agent {}",
            version.version, version.user_agent
        );

        println!("* Receiving verack");
        let response = Envelope::parse(&mut reader).unwrap();

        if response.command() != "verack" {
            println!("Received {} command instead of verack", response.command());
            return Err(());
        }
    }

    let verack = Envelope::new(Network::Main, "verack", vec![]).unwrap();
    println!("* Sending verack");
    match stream.write(&verack.serialize()) {
        Err(err) => println!("Error: {}", err.to_string()),
        Ok(size) => println!("Wrote {} bytes", size),
    }

    {
        let mut reader = BufReader::new(&mut stream);
        println!("* Receiving sendcmpct");
        let response = Envelope::parse(&mut reader).unwrap();

        if response.command() != "sendcmpct" {
            return Err(());
        }

        println!("* Receiving ping");
        let response = Envelope::parse(&mut reader).unwrap();

        println!("* Sending pong");
        let pong = Envelope::new(Network::Main, "pong", response.payload).unwrap();
        match stream.write(&pong.serialize()) {
            Err(err) => println!("Error: {}", err.to_string()),
            Ok(size) => println!("Wrote {} bytes", size),
        }
    }

    let get_headers = Envelope::new(
        Network::Main,
        "getheaders",
        GetHeadersMessage {
            version: 70015,
            num_hashes: 1,
            start_block: BlockId::new(U256::from_hex(
                "000000000000000000020572d673f1b674b3a58f39d235102fd7c2067ccb95a5",
            )),
            end_block: BlockId::new(U256::default()),
        }
        .serialize(),
    )
    .unwrap();

    println!("* Sending get headers message");
    match stream.write(&get_headers.serialize()) {
        Err(err) => println!("Error: {}", err.to_string()),
        Ok(size) => println!("Wrote {} bytes", size),
    }

    println!("* Receiving next message");
    let mut reader = BufReader::new(&mut stream);
    let mut response = Envelope::parse(&mut reader).unwrap();

    while response.command() != "headers" {
        println!("* Received {}", response.command());
        println!("* Receiving next message");
        response = Envelope::parse(&mut reader).unwrap();
    }

    println!("* Parsing headers message");
    let headers = HeadersMessage::parse(&mut response.payload.as_slice()).unwrap();

    let mut previous: Option<BlockId> = None;
    for header in headers.headers {
        println!("** Checking header: {}", header.id());
        println!("*** POW: {}", if header.check_pow() { "OK" } else { "KO" });
        if previous.is_some() {
            println!(
                "*** Previous block: {}",
                if header.prev_block == previous.unwrap() {
                    "OK"
                } else {
                    "KO"
                }
            );
        }
        previous = Some(header.hash());
    }

    Ok(())
}
