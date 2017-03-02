use std::process::exit;
use std::net::{TcpStream, TcpListener};
use std::thread::spawn;
use std::io::Read;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 3 {
    println!("Not enough arguments");
    exit(-1);
  }
//  let src = args[1].parse().unwrap_or_else(|e| { println!("Invalid arguments"); exit(1); });
//  let dst = args[2].parse().unwrap_or_else(|e| { println!("Invalid arguments"); exit(1); });

  let (input, output) = std::sync::mpsc::channel::<Vec<u8>>();

  let server = TcpListener::bind(&*args[1]).unwrap_or_else(|_e| { println!("Failed to bind to listen address {}", args[1]); exit(1); });
//  let client = TcpStream::connect(&*args[2]).unwrap_or_else(|e| { println!("Failed to connect to address {}", args[2]); exit(1); });

  spawn(move || {
    loop {
      match server.accept() { // Might also iterate over incoming() here
        Ok(res) => {
          let (mut stream, addr) = res;
          let my_input = input.clone();
          println!("New incoming connection from {}", addr);
          spawn(move || {
            loop {
              let mut buf = vec![0; 5];
              let n = stream.read(&mut buf).unwrap();
              if n == 0 {
                println!("Connection from {} closed", addr);
                return;
              }
              my_input.send(buf).unwrap();
            }
          });
        }
        Err(_) => panic!("Error in accept()")
      }
    }
  });

  loop {
    println!("Input: {:?}", String::from_utf8(output.recv().unwrap()).unwrap());
  }
  // send to client
}
