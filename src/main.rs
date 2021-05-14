//crates
use std::io::{self, ErrorKind, Read, Write, stdout};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use crossterm::{
  ExecutableCommand, cursor::{MoveUp, MoveLeft},
};

// modules
mod lib;
use crate::lib::crypto::*;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 256;

fn main() {
    println!("Connecting to server...");
    match TcpStream::connect(LOCAL) {
        Ok(client) => {
            client.set_nonblocking(true).expect("failed to initiate non-blocking");

            match validate_keys() {
                Ok(_) => (),
                Err(_) => {
                    println!("Could not validate keys");
                    return ()
                }
            }

            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            spawn_listener_thread(rx, client);
            start_input_loop(tx);

            println!("bye bye!");
        },
        Err(e) => {
            println!("Could not connect to server at {} because of error: \"{}\"", LOCAL, e)
        }
    }
}

fn spawn_listener_thread(rx: mpsc::Receiver<Vec<u8>>, mut client: TcpStream) {
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                match decrypt_message(buff) {
                    Ok(msg) => {
                      stdout().execute(MoveLeft(5000)).expect("failed move cursor");
                      println!("{}", msg)
                    },
                    Err(_e) => {
                        // Problem: We can't decrypt messages we sent, but the server always echos them back
                        // println!("Unable to decrypt message: {}", e);
                        ()
                    }
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(e) => {
                println!("ERROR: {}", e);
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });
}

fn start_input_loop(tx: mpsc::Sender<Vec<u8>>) {
    println!("Choose your username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("reading from stdin failed");
    stdout().execute(MoveUp(1)).expect("failed move cursor");

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let buff = format!("{}: {}", username.trim(), buff.trim()).to_string();
        stdout().execute(MoveUp(1)).expect("failed move cursor");
        println!("{}", buff);
        let msg = encrypt_message(&buff);
        match tx.send(msg) {
            Ok(_) => (),
            Err(e) => {
                println!("Input Error: {}", e);
                break
            }
        }
    }
}
