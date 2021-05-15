pub mod crypto {
    use openssl::rsa::{Rsa, Padding};
    use openssl::sign::{Signer, Verifier};
    use openssl::pkey::{PKey};
    use openssl::hash::MessageDigest;
    use std::fs;

    const SENDER_PUBLIC: &str = "other_test_public.pem";
    const SENDER_PRIVATE: &str = "other_test_private.pem";
    const RECEIVER_PUBLIC: &str = "test_public.pem";
    const RECEIVER_PRIVATE: &str = "test_private.pem";

    pub fn validate_keys() -> Result<(),&'static str> {
        // sign test message with own private key
        let private_key = match fs::read(SENDER_PRIVATE) {
            Ok(key) => key,
            Err(_) => return Err("Could not find private key"),
        };
        let key = PKey::private_key_from_pem(&private_key).unwrap();
        let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
        let msg = "key test".as_bytes();
        signer.update(&msg).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        // verify test message with own public key
        let public_key = match fs::read(RECEIVER_PUBLIC) {
          Ok(key) => key,
          Err(_) => return Err("Could not find public key"),
        };
        let key = PKey::public_key_from_pem(&public_key).unwrap();
        let mut verifier = Verifier::new(MessageDigest::sha256(), &key).unwrap();
        verifier.update(&msg).unwrap();
        match verifier.verify(&signature) {
            Ok(_) => Ok(()),
            Err(_) =>  Err("Could not validate keys"),
        }
    }

    pub fn encrypt_message(msg: &str) -> Vec<u8> {
        let key = fs::read(RECEIVER_PUBLIC).unwrap();
        let rsa = Rsa::public_key_from_pem(&key).unwrap();
        let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.public_encrypt(msg.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
        let mut signature = sign_message(&buf);
        buf.append(&mut signature);
        buf
    }

    pub fn decrypt_message(mut msg: Vec<u8>) -> Result<String, openssl::error::ErrorStack> {
        let signature = msg.split_off(128);
        let key = fs::read(SENDER_PRIVATE).unwrap();
        let rsa = Rsa::private_key_from_pem(&key).unwrap();
        let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
        match rsa.private_decrypt(&msg, &mut buf, Padding::PKCS1) {
            Ok(_) => {
                match verify_message(msg, signature) {
                    Ok(_) => Ok(String::from_utf8(buf).unwrap()),
                    Err(_) => {
                        let mut msg = String::from_utf8(buf).unwrap();
                        msg.push_str("      ****** Unable to verify legitimate sender");
                        Ok(msg)
                    }
                }
            },
            Err(e) => Err(e),
        }
    }

    pub fn sign_message(msg: &Vec<u8>) -> Vec<u8> {
        let key = fs::read(RECEIVER_PRIVATE).unwrap();
        let key = PKey::private_key_from_pem(&key).unwrap();
        let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
        signer.update(&msg).unwrap();
        signer.sign_to_vec().unwrap()
    }

    pub fn verify_message(msg: Vec<u8>, signature: Vec<u8>) -> Result<(), ()> {
        let key = fs::read(SENDER_PUBLIC).unwrap();
        let key = PKey::public_key_from_pem(&key).unwrap();
        let mut verifier = Verifier::new(MessageDigest::sha256(), &key).unwrap();
        verifier.update(&msg).unwrap();
        match verifier.verify(&signature).unwrap() {
            true => Ok(()),
            false => Err(()),
        }
    }
}


