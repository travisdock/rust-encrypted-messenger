# Rust Encrypted Messenger
An encrypted messaging command line app that works via tcp

### Running a Client
 - Download the binary
 - `chmod 755 {FILENAME}`
 - `./{FILENAME}` (Give permission in System Preferences -> Security & Privacy -> General
 - Input your server location. E.g. `X.tcp.ngrok.io:XXXXX`
 - Input a username: `COOL_GUY_42`
 - Start chatting

 ### Creating a Release
- `git tag -a vX.X.X -m "{description}"`
- `git push origin vX.X.X`

### Inspiration & Notes
- Rust TCP Chat: https://github.com/tensor-programming/Rust_client-server_chat
- OpenSSL RSA Encryption: https://rust-by-example-ext.com/openssl/rsa.html
- OpenSSL Signing: https://docs.rs/openssl/0.10.34/openssl/sign/index.html
