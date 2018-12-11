
extern crate amqp_worker;
extern crate log;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;

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

static HTTP_EVENT: HttpEvent = HttpEvent{};

fn main() {
  simple_logger::init_with_level(log::Level::Info).unwrap();

  loop{
    start_worker(&HTTP_EVENT);
  }
}
