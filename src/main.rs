use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, TcpListener};
use std::process::exit;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::thread;
use std::time::{Duration, Instant};

static INPUTS: AtomicUsize = ATOMIC_USIZE_INIT;

pub trait DurationToString {
  fn to_string(self) -> String;
}
impl DurationToString for Duration {
  fn to_string(self) -> String {
    let mut secs = self.as_secs();
    let mut result = String::with_capacity(10);

    if secs == 0 {
      result.push_str("0s");
      return result;
    }

    let delta = [ 31449600, 604800, 86400, 3600, 60, 1 ];
    let unit = [ 'y', 'w', 'd', 'h', 'm', 's' ];
    let mut c = 0;

    loop {
      if secs >= delta[c] { break; }
      c += 1;
    }
    result.push_str(&format!("{}{}", secs/delta[c], unit[c]));
    secs = secs%delta[c];
    if secs != 0 {
      c += 1;
      result.push_str(&format!(" {}{}", secs/delta[c], unit[c]));
    }
    return result;
  }
}

fn connect(addr: &str) -> TcpStream {
  let recon_delay = Duration::new(30, 0);
  println!("Recon delay increment is {}", recon_delay.to_string());
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
  let my_input = input.clone();
  thread::spawn(move || {
    for res in server.incoming() {
      let stream = res.unwrap();
      let addr = stream.peer_addr().unwrap();
      let my_input = input.clone();
      println!("New incoming connection from {}", addr);
      thread::spawn(move || {
        INPUTS.fetch_add(1, Ordering::SeqCst);
        let stream = BufReader::new(stream);
        for line in stream.lines() {
          my_input.send(line.unwrap()).unwrap();
        }
        println!("Connection from {} closed", addr);
        INPUTS.fetch_sub(1, Ordering::SeqCst);
      });
    }
  });

  thread::spawn(move || {
    let delay = Duration::new(300, 0);
    loop {
      thread::sleep(delay);
      my_input.send("\0".to_string()).unwrap();
    }
  });

  let mut lines_written = 0;
  let mut client = connect(&*args[2]);
  let mut conn_since = Instant::now();

  loop {
    let mut line = output.recv().unwrap();
    if line == "\0" {
      println!("{} incoming connections | {} lines relayed | connected for {}", INPUTS.load(Ordering::SeqCst), lines_written, conn_since.elapsed().to_string());
      lines_written = 0;
      continue;
    }
    line.push('\n');
    loop {
      match client.write_all(line.as_bytes()) {
        Ok(_) => {
          lines_written += 1;
          break;
        }
        Err(e) => {
          println!("Write error [{}]; reconnecting...", e.to_string());
          client = connect(&*args[2]);
          conn_since = Instant::now();
        }
      }
    }
  }
}
