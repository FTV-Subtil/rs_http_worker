use amqp_worker::*;
use reqwest;
use reqwest::StatusCode;
use std::fs::File;
use std::io::prelude::*;

pub fn process(message: &str) -> Result<u64, MessageError> {
    let job = job::Job::new(message)?;
    debug!("reveived message: {:?}", job);

    match job.check_requirements() {
        Ok(_) => {}
        Err(message) => {
            return Err(message);
        }
    }

    let source_path = job.get_string_parameter("source_path");
    let destination_path = job.get_string_parameter("destination_path");

    if source_path.is_none() {
        return Err(MessageError::ProcessingError(
            job.job_id,
            "missing source path parameter".to_string(),
        ));
    }

    if destination_path.is_none() {
        return Err(MessageError::ProcessingError(
            job.job_id,
            "missing destination path parameter".to_string(),
        ));
    }

    let url = source_path.unwrap();
    let filename = destination_path.unwrap();

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| MessageError::ProcessingError(job.job_id, e.to_string()))?;

    let mut response = client
        .get(url.as_str())
        .send()
        .map_err(|e| MessageError::ProcessingError(job.job_id, e.to_string()))?;

    let status = response.status();

    if status != StatusCode::OK {
        println!("ERROR {:?}", response);
        return Err(MessageError::ProcessingError(
            job.job_id,
            "bad response status".to_string(),
        ));
    }

    let mut body: Vec<u8> = vec![];
    response
        .copy_to(&mut body)
        .map_err(|e| MessageError::ProcessingError(job.job_id, e.to_string()))?;

    let mut file = File::create(filename.as_str())
        .map_err(|e| MessageError::ProcessingError(job.job_id, e.to_string()))?;
    file.write_all(&body)
        .map_err(|e| MessageError::ProcessingError(job.job_id, e.to_string()))?;

    Ok(job.job_id)
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
        "id": "source_paths",
        "type": "array_of_strings",
        "value": ["https://staticftv-a.akamaihd.net/sous-titres/france4/20180214/172524974-5a843dcd126f8-1518616910.ttml"]
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
        "id": "source_paths",
        "type": "array_of_strings",
        "value": ["https://staticftv-a.akamaihd.net/sous-titres/france4/20180214/172524974-5a843dcd126f8-1518616910.ttml"]
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
    assert_eq!(
        result,
        Err(MessageError::RequirementsError(
            "Warning: Required file does not exists: \"/tmp/FiLe_ThAt_$h0uld_N0t_3xist$\""
                .to_string()
        ))
    );
}
