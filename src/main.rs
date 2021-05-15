use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, PartialEq, Eq)]
enum Request {
    Publish(String),
    Retrieve,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let storage = Arc::new(Mutex::new(VecDeque::new()));

    for connection_attepmt in listener.incoming() {
        match connection_attepmt {
            Ok(stream) => {
                let thread_handle = Arc::clone(&storage); //storage.clone();
                std::thread::spawn(move || {
                    handle_client(stream, &thread_handle);
                });
            }
            Err(e) => {
                eprintln!("Error connecting: {}", e)
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, storage: &Mutex<VecDeque<String>>) {
    let line = read_line(&stream);
    let request = parse_request(line);

    match request {
        Request::Publish(msg) => {
            let mut guard = storage.lock().unwrap();
            guard.push_back(msg);
            println!("publishing message")
        }
        Request::Retrieve => {
            // let mut guard = storage.lock().unwrap(); 
            // guard lives after match, not needed that long
            let maybe_msg = storage.lock().unwrap().pop_front();
            match maybe_msg {
                Some(msg) => {
                    stream.write_all(msg.as_bytes()).unwrap();
                }
                None => {
                    stream.write_all(b"no message available\n").unwrap();
                }
            }
            println!("retriveving message")
        }
    }
    println!("Client connected!");
}

fn parse_request(line: String) -> Request {
    let trimmed = line.trim_end();

    if trimmed.is_empty() {
        Request::Retrieve
    } else {
        let s = format!("FIFO: {}\n", trimmed);
        Request::Publish(s)
    }
}

fn read_line(stream: &TcpStream) -> String {
    let mut buffered_reader = BufReader::new(stream);

    let mut buf = String::new();
    buffered_reader.read_line(&mut buf).unwrap();

    buf
}
