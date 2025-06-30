use std::{env::{self, args}, io::{Read, Write}, net::{Ipv4Addr, SocketAddrV4, TcpStream}, str::FromStr, thread, time::Duration};

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let port = args.next().unwrap().parse().unwrap();

    let addr = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1").unwrap(), port);
    let mut stream = TcpStream::connect(addr).unwrap();

    loop{
        let mut buf = [0;4096];
        let n = stream.read(&mut buf).expect("error on stream.read");
        let string = str::from_utf8(&buf[..n]).unwrap().to_string();
        println!("AGENT 4 GOT '{string}'");

        thread::sleep(Duration::from_millis(500)); //Simulate some computation

        let size = stream.write(1.to_string().as_bytes()).expect("AGENT 4: WRITE ERROR");
        assert!(size == 1.to_string().as_bytes().len());
    } 
}
