
extern crate amqp_worker;
extern crate log;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;

use amqp_worker::*;
use std::env;
use log::Level;

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
  if let Ok(_)= env::var("VERBOSE") {
    simple_logger::init_with_level(Level::Debug).unwrap();
  } else {
    simple_logger::init_with_level(Level::Warn).unwrap();
  }

  loop {
    let http_event = HttpEvent{};
    start_worker(&http_event);
  }
}
