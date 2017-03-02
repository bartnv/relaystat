use std::process::exit;
use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 3 {
    println!("Not enough arguments");
    exit(-1);
  }

  let server = TcpListener::bind(&*args[1]).unwrap_or_else(|e| { println!("Failed to bind to listen address {}: {}", args[1], e.to_string()); exit(1); });

  let (input, output) = std::sync::mpsc::channel();
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

  let mut client = TcpStream::connect(&*args[2]).unwrap_or_else(|e| { println!("Failed to connect to address {}: {}", args[2], e.to_string()); exit(1); });
  println!("Outgoing connection to {} established", args[2]);
  let mut lines_written = 0;
  let recon_delay = std::time::Duration::new(60, 0);

  loop {
    let mut line = output.recv().unwrap();
    line.push('\n');
    match client.write_all(line.as_bytes()) {
      Ok(_) => {
        lines_written += 1;
        if lines_written%100 == 0 { println!("{} lines received", lines_written); }
      }
      Err(e) => {
        println!("Write error to {} ({}); reconnecting...", client.peer_addr().unwrap(), e.to_string());
        thread::sleep(recon_delay);
        match TcpStream::connect(&*args[2]) {
          Ok(stream) => { client = stream; }
          Err(e) => { println!("Failed to connect to address {}: {}", args[2], e.to_string()); }
        }
      }
    }
  }
}
