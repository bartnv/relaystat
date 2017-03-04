use std::process::exit;
use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

static GLOBAL_THREAD_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

fn connect(addr: &str) -> TcpStream {
  let recon_delay = Duration::new(30, 0);
  loop {
    match TcpStream::connect(addr) {
      Ok(stream) => {
        println!("Outgoing connection to {} established", addr);
        return stream;
      }
      Err(e) => println!("Failed to connect to {}: {}", addr, e.to_string())
    }
    recon_delay + Duration::new(30, 0);
    println!("Delaying reconnect for {:?}", recon_delay);
    thread::sleep(recon_delay);
  }
}

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
        GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        let stream = BufReader::new(stream);
        for line in stream.lines() {
          my_input.send(line.unwrap()).unwrap();
        }
        println!("Connection from {} closed", addr);
        GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
      });
    }
  });

  let mut client = connect(&*args[2]);
  let mut lines_written = 0;

  loop {
    let mut line = output.recv().unwrap();
    line.push('\n');
    loop {
      match client.write_all(line.as_bytes()) {
        Ok(_) => {
          lines_written += 1;
          if lines_written%100 == 0 { println!("{:?} incoming connections | {} lines received", GLOBAL_THREAD_COUNT, lines_written); }
          break;
        }
        Err(e) => {
          println!("Write error [{}]; reconnecting...", e.to_string());
          client = connect(&*args[2]);
        }
      }
    }
  }
}
