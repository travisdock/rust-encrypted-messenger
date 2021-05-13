use std::io::{self, ErrorKind, Read, Write, stdout};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use crossterm::{
  ExecutableCommand, cursor::{MoveUp, MoveDown},
};

// Crypto stuff
use openssl::rsa::{Rsa, Padding};
use std::fs::File;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;
const USERNAME: &str = "The DUDE";

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                // let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                stdout().execute(MoveDown(1)).expect("failed move cursor");
                println!("{:?}", msg);
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
                // println!("message sent {:?}", msg);
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
        stdout().execute(MoveUp(2)).expect("failed move cursor");
        // let msg = format!("{}: {}", USERNAME, buff.trim()).to_string();
        let msg = encrypt_message(&buff);
        tx.send(msg).expect("You fucked up");
    }
    println!("bye bye!");

}


fn encrypt_message(msg: &str) -> Vec<u8> {
    let mut file = File::open("test_public.pem").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let rsa = Rsa::public_key_from_pem(contents.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    rsa.public_encrypt(msg.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
    buf
}

// fn decrypt_message(msg: Message, passphrase: &str) -> String {
//     let mut file = File::open("test_private.pem").unwrap();
//     let mut contents = String::new();
//     file.read_to_string(&mut contents).unwrap();
//     let rsa = Rsa::private_key_from_pem_passphrase(contents.as_bytes(), passphrase.as_bytes()).unwrap();
//     let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
//     rsa.private_decrypt(&msg.body, &mut buf, Padding::PKCS1).unwrap();
//     // println!("Decrypted: {}", String::from_utf8(buf).unwrap());
//     String::from_utf8(buf).unwrap()
// }
