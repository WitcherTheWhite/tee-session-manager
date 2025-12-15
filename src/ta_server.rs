use bincode::{Decode, Encode, config};
use dashmap::DashSet;
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    sync::Arc,
    thread, time::Duration,
};

use crate::protocal::{CARequest, Parameters};

#[derive(Encode, Decode)]
enum TARequest {
    Register { uuid: String },
}

#[derive(Encode, Decode)]
enum TAResponse {
    Ok,
    Err(String),
}

pub fn handle_ta_request(
    mut stream: UnixStream,
    registry: Arc<DashSet<String>>,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 1024];

    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }

    let config = config::standard();
    let (req, _): (TARequest, usize) = bincode::decode_from_slice(&buf, config)?;

    match req {
        TARequest::Register { uuid } => {
            registry.insert(uuid.clone());
            println!("Connected to TA socket");

            let path = format!("/tmp/{}.sock", uuid);
            let path_clone = path.clone();

            let handle1 = thread::spawn(move || {
                let mut stream = UnixStream::connect(path_clone).unwrap();
                let req = CARequest::OpenSession {
                    params: Parameters::default(),
                };
                let encoded = bincode::encode_to_vec(req, config).unwrap();
                let mut message = Vec::with_capacity(4 + encoded.len());

                // 方法1: 使用 byteorder
                message.extend_from_slice(&(encoded.len() as u32).to_ne_bytes());

                message.extend_from_slice(&encoded);
                stream.write_all(&message).unwrap();
                stream.write_all(&encoded).unwrap();
            });

            thread::sleep(Duration::from_secs(1));

            let handle2 = thread::spawn(move || {
                let mut stream = UnixStream::connect(path).unwrap();
                let req = CARequest::InvokeCommand {
                    session_id: 1,
                    cmd_id: 0,
                    params: Parameters::default(),
                };
                let encoded = bincode::encode_to_vec(req, config).unwrap();
                let mut message = Vec::with_capacity(4 + encoded.len());

                // 方法1: 使用 byteorder
                message.extend_from_slice(&(encoded.len() as u32).to_ne_bytes());

                message.extend_from_slice(&encoded);
                stream.write_all(&message).unwrap();
                stream.write_all(&encoded).unwrap();
            });

            handle1.join().unwrap();
            handle2.join().unwrap();
        }
        _ => {
            let resp = TAResponse::Err("Unsupported operation".to_string());
            let encoded = bincode::encode_to_vec(resp, config)?;
            stream.write_all(&encoded)?;
        }
    }

    Ok(())
}
