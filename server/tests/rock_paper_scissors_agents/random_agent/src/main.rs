use std::{
    env,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str::FromStr,
};

use rand::seq::IndexedRandom;

use crate::games::RpsAction::{Paper, Rock, Scissors};

mod games;

fn find_action(_state:&games::PlayerState) -> games::RpsAction {
    let mut rng = rand::rng();
    *[Rock,Paper,Scissors].choose(&mut rng).unwrap()
}

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let port = args.next().unwrap().parse().unwrap();

    let addr = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1").unwrap(), port);
    let mut stream = TcpStream::connect(addr).unwrap();

    loop {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf).expect("error on stream.read");
        let string = str::from_utf8(&buf[..n]).unwrap();
        
        println!("AGENT GOT '{string}'");
        let state = games::PlayerState::from_str(string).expect("from_str error (agent)");

        let action = find_action(&state);

        let action_str = action.to_string();

        stream.write_all(action_str.as_bytes()).expect("could not send (write error)");
    }
}
