use bitlib::definitions::Network;
use bitlib::hashing::to_hex_str;
use bitlib::network::{Envelope, NetAddr, VersionMessage};
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
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

    match stream.write(&version.serialize()) {
        Err(err) => println!("Error: {}", err.to_string()),
        Ok(size) => println!("Wrote {} bytes", size),
    }

    let mut reader = BufReader::new(&stream);
    let response = Envelope::parse(&mut reader).unwrap();
    println!("{}", response);

    let version = VersionMessage::parse(&mut response.payload.as_slice()).unwrap();
    println!("{}", version);

    let response = Envelope::parse(&mut reader).unwrap();
    println!("{}", response);

    if response.command() != "verack" {
        return Err(());
    }

    let verack = Envelope::new(Network::Main, "verack", vec![]).unwrap();

    match stream.write(&verack.serialize()) {
        Err(err) => println!("Error: {}", err.to_string()),
        Ok(size) => println!("Wrote {} bytes", size),
    }

    Ok(())
}
