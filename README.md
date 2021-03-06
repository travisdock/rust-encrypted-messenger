# Rust Encrypted Messenger
An encrypted and signed messaging command line app that works via tcp

<img width="511" alt="Screen Shot 2021-05-18 at 6 51 05 PM" src="https://user-images.githubusercontent.com/36681963/118740879-3008d100-b80a-11eb-8a99-09dabc7e5f29.png">



### Running a Client (OSX)
 - Download the binary from the releases page
 - `chmod 755 BINARY_FILENAME`
 - Create a `private.pem` and `public.pem` key file (https://travistidwell.com/jsencrypt/demo/)
 - Retrieve the public key of your fellow chatter in a `recipient_public.pem` file (and give them yours)
 - Place all keys in the same directory as your binary
 - Make sure you have a server running (https://github.com/travisdock/Rust-TCP-server)
 - Run the binary: `./BINARY_FILENAME` (If necessary give permission in System Preferences -> Security & Privacy -> General)
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
