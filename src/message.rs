
use reqwest;
use reqwest::StatusCode;
use serde_json;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct Resource {
  path: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Requirement {
  paths: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
struct Parameters {
  requirement: Requirement,
  source: Resource,
  destination: Resource
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
  job_id: u64,
  parameters: Parameters
}

pub enum MessageError {
  RuntimeError(String),
  RequirementError(String)
}

fn check_requirements(requirements: Requirement) -> Result<(), MessageError> {
  if requirements.paths.is_some() {
    let required_paths :Vec<String> = requirements.paths.unwrap();
    for path in &required_paths {
      let p = Path::new(path);
      if !p.exists() {
        return Err(MessageError::RequirementError(format!("Warning: Required file does not exists: {:?}", p)));
      }
    }
  }
  Ok(())
}

pub fn process(message: &str) -> Result<u64, MessageError> {

  let parsed: Result<Job, _> = serde_json::from_str(message);

  match parsed {
    Ok(content) => {
      println!("{:?}", content);

      let parameters = content.parameters;

      match check_requirements(parameters.requirement) {
        Ok(_) => {},
        Err(message) => { return Err(message); }
      }

      let url = parameters.source.path;
      let filename = parameters.destination.path;

      let client = reqwest::Client::builder()
        .build()
        .unwrap();

      let mut response = client.get(url.as_str()).send().unwrap();

      let status = response.status();

      if !(status == StatusCode::Ok) {
        println!("ERROR {:?}", response);
        return Err(MessageError::RuntimeError("bad response status".to_string()));
      }

      let mut body: Vec<u8> = vec![];
      response.copy_to(&mut body).unwrap();

      let mut file = File::create(filename.as_str()).unwrap();
      file.write_all(&body).unwrap();

      Ok(content.job_id)
    },
    Err(msg) => {
      println!("ERROR {:?}", msg);
      return Err(MessageError::RuntimeError("bad input message".to_string()));
    }
  }
}

#[test]
fn ack_message_test() {

  let message = "{ \
      \"parameters\":{ \
        \"requirement\":{ \
        }, \
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

#[test]
fn nack_message_test() {

  let message = "{ \
      \"parameters\":{ \
        \"requirement\":{ \
          \"paths\": [\"/tmp/FiLe_ThAt_$h0uld_N0t_3xist$\"] \
        }, \
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
  assert!(result.is_err());
}
