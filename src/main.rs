use std::io::{self, ErrorKind, Read, Write, stdout};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::fs;
use crossterm::{
  ExecutableCommand, cursor::{MoveUp, MoveLeft},
};
use rpassword::read_password;

// Crypto stuff
use openssl::rsa::{Rsa, Padding};
use openssl::sign::{Signer, Verifier};
use openssl::pkey::{PKey, Private as PrivateKey};
use openssl::hash::MessageDigest;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 256;
const USERNAME: &str = "The DUDE";

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");
    let passphrase = authenticate_key();
    let send_passphrase = passphrase.clone();

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                // let passphrase = String::from("rust_by_example");
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
        let msg = encrypt_message(&buff, &send_passphrase);
        tx.send(msg).expect("You fucked up");
    }
    println!("bye bye!");
}

fn authenticate_key() -> String {
    // get passphrase
    println!("Please enter your private key passphrase:");
    let passphrase = read_password().expect("Unable to get passphrase");
    let passbytes = &passphrase.as_bytes();

    // sign test message with private key
    let private_key = fs::read("test_private.pem").expect("Unable to find private key");
    let key = PKey::private_key_from_pem_passphrase(&private_key, passbytes).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
    let msg = "passphrase test".as_bytes();
    signer.update(&msg).unwrap();
    let signature = signer.sign_to_vec().unwrap();

    // verify test message with public key
    let public_key = fs::read("test_public.pem").expect("Unable to find public key");
    let key = PKey::public_key_from_pem(&public_key).unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &key).unwrap();
    verifier.update(&msg).unwrap();
    verifier.verify(&signature).unwrap();
    passphrase
}


fn encrypt_message(msg: &str, passphrase: &str) -> Vec<u8> {
    let key = fs::read("other_test_public.pem").unwrap();
    let rsa = Rsa::public_key_from_pem(&key).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    rsa.public_encrypt(msg.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
    let mut signature = sign_message(&buf, passphrase);
    buf.append(&mut signature);
    buf
}

fn decrypt_message(mut msg: Vec<u8>, passphrase: &str) -> Result<String, openssl::error::ErrorStack> {
    let signature = msg.split_off(128);
    let key = fs::read("test_private.pem").unwrap();
    let rsa = Rsa::private_key_from_pem_passphrase(&key, passphrase.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    match rsa.private_decrypt(&msg, &mut buf, Padding::PKCS1) {
        Ok(_) => {
            verify_message(msg, signature);
            Ok(String::from_utf8(buf).unwrap())
        },
        Err(e) => Err(e),
    }
}

fn sign_message(msg: &Vec<u8>, passphrase: &str) -> Vec<u8> {
  let key = fs::read("test_private.pem").unwrap();
  let key = PKey::private_key_from_pem_passphrase(&key, passphrase.as_bytes()).unwrap();
  let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
  signer.update(&msg).unwrap();
  signer.sign_to_vec().unwrap()
}

fn verify_message(msg: Vec<u8>, signature: Vec<u8>) {
    let key = fs::read("test_public.pem").unwrap();
    let key = PKey::public_key_from_pem(&key).unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &key).unwrap();
    verifier.update(&msg).unwrap();
    assert!(verifier.verify(&signature).unwrap());
}
