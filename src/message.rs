
use reqwest;
use reqwest::StatusCode;
use serde_json;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct Resource {
  path: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Parameters {
  source: Resource,
  destination: Resource
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
  job_id: u64,
  parameters: Parameters
}

pub fn process(message: &str) -> Result<u64, &str> {

  let parsed: Result<Job, _> = serde_json::from_str(message);

  match parsed {
    Ok(content) => {
      println!("{:?}", content);
      let url = content.parameters.source.path;
      let filename = content.parameters.destination.path;

      let client = reqwest::Client::builder()
        .build()
        .unwrap();

      let mut response = client.get(url.as_str()).send().unwrap();

      let status = response.status();

      if !(status == StatusCode::Ok) {
        println!("ERROR {:?}", response);
        return Err("bad response status");
      }

      let mut body: Vec<u8> = vec![];
      response.copy_to(&mut body).unwrap();

      let mut file = File::create(filename.as_str()).unwrap();
      file.write_all(&body).unwrap();

      Ok(content.job_id)
    },
    Err(msg) => {
      println!("ERROR {:?}", msg);
      return Err("bad input message");
    }
  }
}

#[test]
fn message_test() {

  let message = "{ \
      \"parameters\":{ \
        \"source\":{ \
          \"path\":\"https://staticftv-a.akamaihd.net/sous-titres/france4/20180214/172524974-5a843dcd126f8-1518616910.ttml\" \
        }, \
        \"destination\":{ \
          \"path\":\"/tmp/ftp_ftv/97d4354b-9a2b-4ef9-ba43-b6c422bd989e/172524974-5a843dcd126f8-1518616910.ttml\" \
        } \
      }, \
      \"job_id\":690 \
    } \
    ";

  let result = process(message);
  assert!(result.is_ok());
}
