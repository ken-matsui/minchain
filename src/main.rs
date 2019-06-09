use std::net::TcpListener;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::io::{Read, Write};
use std::io;
use std::str::from_utf8;

fn handle_message(msg: &[u8]) {
    println!("{}", from_utf8(&msg.to_vec()).unwrap());
}

fn server_start() -> io::Result<()> {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let port: u16 = 50030;

    println!("My IP address is now ... {}", localhost);
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(localhost), port))?;

    loop {
        println!("Waiting for the connection ...");
        match listener.accept() {
            Ok((mut stream, addr)) => {
                println!("Connected by ... {}", addr);
                let _ = thread::spawn(
                    move || -> io::Result<()> {
                        loop {
                            let mut b = [0; 1024];
                            let n = stream.read(&mut b)?;
                            if n == 0 {
                                return Ok(());
                            } else {
                                handle_message(&b[0..n]);
                                // stream.write(&b[0..n])?;Z
                            }
                        }
                    }
                );
            },
            Err(e) => {
                println!("An error occurred while accepting a connection: {}", e);
                continue;
            }
        };
    }
}

fn main() {
    match server_start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
