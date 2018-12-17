
use amqp_worker::*;
use reqwest;
use reqwest::StatusCode;
use serde_json;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Parameter {
  #[serde(rename = "string")]
  StringParam{id: String, default: Option<String>, value: Option<String>},
  #[serde(rename = "paths")]
  PathsParam{id: String, default: Option<Vec<String>>, value: Option<Vec<String>>},
  #[serde(rename = "requirements")]
  RequirementParam{id: String, default: Option<Requirement>, value: Option<Requirement>},
}

#[derive(Debug, Serialize, Deserialize)]
struct Requirement {
  paths: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
  job_id: u64,
  parameters: Vec<Parameter>
}

fn get_parameter(params: &Vec<Parameter>, key: &str) -> Option<String> {
  for param in params.iter() {
    match param {
      Parameter::StringParam{id, default, value} => {
        if id == key {
          if let Some(ref v) = value {
            return Some(v.clone())
          } else {
            return default.clone()
          }
        }
      },
      _ => {}
    }
  }
  None
}

fn check_requirements(params: &Vec<Parameter>) -> Result<(), MessageError> {
  for param in params.iter() {
    match param {
      Parameter::RequirementParam{id, value, ..} => {
        if id == "requirements" {
          if let Some(Requirement{paths: Some(paths)}) = value {
            for ref path in paths.iter() {
              let p = Path::new(path);
              if !p.exists() {
                return Err(MessageError::RequirementsError(format!("Warning: Required file does not exists: {:?}", p)));
              }
            }
          }
        }
      },
      _ => {}
    }
  }
  Ok(())
}

pub fn process(message: &str) -> Result<u64, MessageError> {
  println!("{}", message);
  let parsed: Result<Job, _> = serde_json::from_str(message);

  match parsed {
    Ok(content) => {
      println!("{:?}", content);

      match check_requirements(&content.parameters) {
        Ok(_) => {},
        Err(message) => { return Err(message); }
      }

      let url =
        if let Some(source_path) = get_parameter(&content.parameters, "source_path") {
          source_path
        } else {
          return Err(MessageError::ProcessingError(content.job_id, "missing source path parameter".to_string()));
        };
      let filename =
        if let Some(destination_path) = get_parameter(&content.parameters, "destination_path") {
          destination_path
        } else {
          return Err(MessageError::ProcessingError(content.job_id,"missing destination path parameter".to_string()));
        };

      let client = reqwest::Client::builder()
        .build()
        .map_err(|e| MessageError::ProcessingError(content.job_id, e.to_string()))?;

      let mut response = client.get(url.as_str()).send().map_err(|e| MessageError::ProcessingError(content.job_id, e.to_string()))?;

      let status = response.status();

      if !(status == StatusCode::OK) {
        println!("ERROR {:?}", response);
        return Err(MessageError::ProcessingError(content.job_id, "bad response status".to_string()));
      }

      let mut body: Vec<u8> = vec![];
      response.copy_to(&mut body).map_err(|e| MessageError::ProcessingError(content.job_id, e.to_string()))?;

      let mut file = File::create(filename.as_str()).map_err(|e| MessageError::ProcessingError(content.job_id, e.to_string()))?;
      file.write_all(&body).map_err(|e| MessageError::ProcessingError(content.job_id, e.to_string()))?;

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
  let msg = r#"{
    "parameters": [
      {
        "id": "requirements",
        "type": "requirements",
        "value": {"paths": []}
      },
      {
        "id": "source_path",
        "type": "string",
        "value": "https://staticftv-a.akamaihd.net/sous-titres/france4/20180214/172524974-5a843dcd126f8-1518616910.ttml"
      },
      {
        "id": "destination_path",
        "type": "string",
        "value": "/tmp/172524974-5a843dcd126f8-1518616910.ttml"
      }
    ],
    "job_id":690
  }"#;

  let result = process(msg);
  assert!(result.is_ok());
}

#[test]
fn nack_message_test() {
  let msg = r#"{
    "parameters": [
      {
        "id": "requirements",
        "type": "requirements",
        "value": {"paths": [
          "/tmp/FiLe_ThAt_$h0uld_N0t_3xist$"
        ]}
      },
      {
        "id": "source_path",
        "type": "string",
        "value": "https://staticftv-a.akamaihd.net/sous-titres/france4/20180214/172524974-5a843dcd126f8-1518616910.ttml"
      },
      {
        "id": "destination_path",
        "type": "string",
        "value": "/tmp/172524974-5a843dcd126f8-1518616910.ttml"
      }
    ],
    "job_id":690
  }"#;


  let result = process(msg);
  assert_eq!(result, Err(MessageError::RequirementsError("Warning: Required file does not exists: \"/tmp/FiLe_ThAt_$h0uld_N0t_3xist$\"".to_string())));
}
