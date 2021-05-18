pub mod crypto {
    use openssl::rsa::{Rsa, Padding};
    use openssl::sign::{Signer, Verifier};
    use openssl::pkey::{PKey};
    use openssl::hash::MessageDigest;
    use std::fs;

    pub fn validate_keys(
        sender_private_path: &str,
        sender_public_path: &str
    ) -> Result<(),String> {
        // sign test message with own private key
        let msg = "key test".as_bytes().to_vec();
        let signature = match sign_message(&msg, sender_private_path) {
            Ok(signature) => signature,
            Err(e) => return Err(e),
        };

        // verify test message with own public key
        match verify_message(msg, signature, sender_public_path) {
            Ok(true) => Ok(()),
            Ok(false) => Err("Could not validate keys".to_owned()),
            Err(e) =>  Err(e),
        }
    }

    pub fn encrypt_message(
        msg: &str,
        receiver_public_path: &str,
        sender_private_path: &str
    ) -> Result<Vec<u8>,String> {
        let key = fs::read(receiver_public_path).unwrap();
        let rsa = Rsa::public_key_from_pem(&key).unwrap();
        let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.public_encrypt(msg.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
        let mut signature = match sign_message(&buf, sender_private_path) {
            Ok(signature) => signature,
            Err(e) => return Err(e),
        };
        buf.append(&mut signature);
        Ok(buf)
    }

    pub fn decrypt_message(
        mut msg: Vec<u8>,
        sender_private_path: &str,
        receiver_public_path: &str
    ) -> Result<String, openssl::error::ErrorStack> {
        let signature = msg.split_off(128);
        let key = fs::read(sender_private_path).unwrap();
        let rsa = Rsa::private_key_from_pem(&key).unwrap();
        let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
        match rsa.private_decrypt(&msg, &mut buf, Padding::PKCS1) {
            Ok(_) => {
                match verify_message(msg, signature, receiver_public_path) {
                    Ok(true) => Ok(String::from_utf8(buf).unwrap()),
                    _ => {
                        let mut msg = String::from_utf8(buf).unwrap();
                        msg.push_str("      ****** Unable to verify legitimate sender");
                        Ok(msg)
                    },
                }
            },
            Err(e) => Err(e),
        }
    }

    pub fn sign_message(
        msg: &Vec<u8>,
        sender_private_path: &str
    ) -> Result<Vec<u8>,String> {
        let private_key = match fs::read(sender_private_path) {
            Ok(file) => PKey::private_key_from_pem(&file).unwrap(),
            Err(_) => return Err("Could not find private key".to_owned()),
        };
        let mut signer = Signer::new(MessageDigest::sha256(), &private_key).unwrap();
        signer.update(&msg).unwrap();
        match signer.sign_to_vec() {
            Ok(signature) => Ok(signature),
            Err(_) => Err("Unable to create signature".to_owned()),
        }
    }

    pub fn verify_message(
        msg: Vec<u8>,
        signature: Vec<u8>,
        receiver_public_path: &str
    ) -> Result<bool, String> {
        let public_key = match fs::read(receiver_public_path) {
            Ok(file) => PKey::public_key_from_pem(&file).unwrap(),
            Err(_) => return Err("Could not find public key".to_owned()),
        };
        let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).unwrap();
        verifier.update(&msg).unwrap();
        match verifier.verify(&signature).unwrap() {
            bool => Ok(bool),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::crypto::{validate_keys, encrypt_message, decrypt_message};
    const TEST_SENDER_PUBLIC: &str = "src/test_keys/public.pem";
    const TEST_SENDER_PRIVATE: &str = "src/test_keys/private.pem";
    const RANDOM_PUBLIC: &str = "src/test_keys/random_public.pem";
    const RANDOM_PRIVATE: &str = "src/test_keys/random_private.pem";

    #[test]
    fn private_key_error_message() {
        match validate_keys("invalid_private.pem", TEST_SENDER_PUBLIC) {
            Ok(_) => assert!(false, "Should error"),
            Err(e) => assert_eq!(e, "Could not find private key")
        }
    }

    #[test]
    fn public_key_error_message() {
      match validate_keys(TEST_SENDER_PRIVATE, "invalid_public.pem") {
          Ok(_) => assert!(false, "Should error"),
          Err(e) => assert_eq!(e, "Could not find public key")
      }
    }

    #[test]
    fn invalid_keys_error_message() {
      match validate_keys(TEST_SENDER_PRIVATE, RANDOM_PUBLIC) {
          Ok(_) => assert!(false, "Should error"),
          Err(e) => assert_eq!(e, "Could not validate keys")
      }
    }
}


