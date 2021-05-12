pub struct Message {
  pub to: String,
  pub from: String,
  pub body: Vec<u8>,
}

impl Message {
  pub fn new(to: String, from: String, body: Vec<u8>) -> Message {
    Message { to, from, body }
  }
}
