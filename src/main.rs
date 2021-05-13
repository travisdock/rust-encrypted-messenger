use std::io::{self, ErrorKind, Read, Write, stdout};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::fs;
use crossterm::{
  ExecutableCommand, cursor::{MoveUp, MoveLeft},
};

// Crypto stuff
use openssl::rsa::{Rsa, Padding};
use openssl::sign::{Signer, Verifier};
use openssl::pkey::{PKey, Private as PrivateKey};
use openssl::hash::MessageDigest;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 128;
const USERNAME: &str = "The DUDE";

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let passphrase = String::from("rust_by_example");
                match decrypt_message(buff, &passphrase) {
                    Ok(msg) => {
                      stdout().execute(MoveLeft(5000)).expect("failed move cursor");
                      println!("{}", msg)
                    },
                    Err(_) => (),
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

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let buff = format!("{}: {}", USERNAME, buff.trim()).to_string();
        stdout().execute(MoveUp(1)).expect("failed move cursor");
        println!("{}", buff);
        let msg = encrypt_message(&buff);
        tx.send(msg).expect("You fucked up");
    }
    println!("bye bye!");
}


fn encrypt_message(msg: &str) -> Vec<u8> {
    let key = fs::read("other_test_public.pem").unwrap();
    let rsa = Rsa::public_key_from_pem(&key).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    rsa.public_encrypt(msg.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
    buf
}

fn decrypt_message(msg: Vec<u8>, passphrase: &str) -> Result<String, openssl::error::ErrorStack> {
    let key = fs::read("test_private.pem").unwrap();
    let rsa = Rsa::private_key_from_pem_passphrase(&key, passphrase.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    match rsa.private_decrypt(&msg, &mut buf, Padding::PKCS1) {
        Ok(_) => Ok(String::from_utf8(buf).unwrap()),
        Err(e) => Err(e),
    }
}

// fn sign_message(msg: Vec<u8>) {
//   let key = fs::read("test_public.pem").unwrap();
//   let key = PKey::private_key_from_pem(&key).unwrap();
//   let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
//   signer.update(&msg).unwrap();
//   let signature = signer.sign_to_vec().unwrap();
// }

// fn verify_message(msg: Vec<u8>) {
//     let key = fs::read("test_public.pem").unwrap();
//     let key = PKey::public_key_from_pem(&key).unwrap();
//     let mut verifier = Verifier::new(MessageDigest::sha256(), &key).unwrap();
//     verifier.update(&msg).unwrap();
//     assert!(verifier.verify(&signature).unwrap());
// }
