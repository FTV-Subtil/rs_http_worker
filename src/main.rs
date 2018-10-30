
extern crate amqp_worker;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use amqp_worker::*;

mod message;

#[derive(Debug)]
struct HttpEvent {
}

impl MessageEvent for HttpEvent {
  fn process(&self, message: &str) -> Result<u64, MessageError> {
    message::process(message)
  }
}

fn main() {
  let http_event = HttpEvent{};
  start_worker(&http_event);
}
