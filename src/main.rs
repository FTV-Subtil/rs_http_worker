extern crate amqp_worker;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate simple_logger;

use amqp_worker::*;
use log::Level;
use std::env;

mod message;

#[derive(Debug)]
struct HttpEvent {}

impl MessageEvent for HttpEvent {
  fn process(&self, message: &str) -> Result<job::JobResult, MessageError> {
    message::process(message)
  }
}

static HTTP_EVENT: HttpEvent = HttpEvent {};

fn main() {
  println!("HTTP Worker (version {}) started ", env::var("VERSION").expect("missing softwareversion"));
  if env::var("VERBOSE").is_ok() {
    simple_logger::init_with_level(Level::Debug).unwrap();
  } else {
    simple_logger::init_with_level(Level::Warn).unwrap();
  }

  start_worker(&HTTP_EVENT);
}
