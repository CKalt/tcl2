use std::io::prelude::*;
use std::net::TcpStream;
use serde_json::Value;

const REQUEST_PORT: u32 = 8080;
const RESPONSE_PORT: u32 = 8081;

fn get_url(port: u32) -> String {
    format!("localhost:{}", port)
}

fn handle_client(mut request_stream: TcpStream) -> std::io::Result<()> {
    let sample_request = r#"
{
  "command": "RSHOT",
  "objectType": "eventJSON",
  "siteCode": "ds12", 
  "holeNumber": "hole3",  
  "archiveFilename": "Archive_211001_140321", 
  "archivePath": "./path/to/archive/file/"
}
"#;

    let r_json = serde_json::from_str::<Value>(sample_request).unwrap();
    let json_text = serde_json::to_string(&r_json).unwrap();
    println!("request={}", json_text);

    let len_msg = format!("{:08x}", json_text.len());
    let wresult = request_stream.write(len_msg.as_bytes());
    match wresult {
        Err(e) => {
            println!("error writing len_msg: {}", e);
        }
        _ => {}
    }

    let wresult = request_stream.write(json_text.as_bytes());
    match wresult {
        Err(e) => {
            println!("error writing json_text: {}", e);
        }
        _ => {}
    }

    let mut response_stream = TcpStream::connect(get_url(RESPONSE_PORT)).unwrap();
    let mut buffer = String::new();
    response_stream.read_to_string(&mut buffer)?;
    println!("{}", buffer);

    Ok(())
}

fn main() -> std::io::Result<()> {
    let request_stream = TcpStream::connect(get_url(REQUEST_PORT)).unwrap();
    handle_client(request_stream)?;
    Ok(())
}
