use std::process::exit;
use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::BufRead;
use std::io::BufReader;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 3 {
    println!("Not enough arguments");
    exit(-1);
  }

  let (input, output) = std::sync::mpsc::channel();

  let server = TcpListener::bind(&*args[1]).unwrap_or_else(|e| { println!("Failed to bind to listen address {}: {}", args[1], e.to_string()); exit(1); });
//  let client = TcpStream::connect(&*args[2]).unwrap_or_else(|e| { println!("Failed to connect to address {}", args[2]); exit(1); });

  thread::spawn(move || {
    for res in server.incoming() {
      let stream = res.unwrap();
      let addr = stream.peer_addr().unwrap();
      let my_input = input.clone();
      println!("New incoming connection from {}", addr);
      thread::spawn(move || {
        let stream = BufReader::new(stream);
        for line in stream.lines() {
          my_input.send(line.unwrap()).unwrap();
        }
        println!("Connection from {} closed", addr);
      });
    }
  });

  loop {
    println!("Input: {}", output.recv().unwrap());
    // send to client
  }
}
