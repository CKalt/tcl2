use std::io::prelude::*;
use std::net::TcpStream;
use serde_json::Value;
use structopt::StructOpt;
use std::fs;

const DEFAULT_REQUEST_PORT:  &str = "8080";
const DEFAULT_RESPONSE_PORT: &str = "8081";

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(default_value = DEFAULT_REQUEST_PORT, short = "i", long = "input-port")]
    input_port: u32,
    #[structopt(default_value = DEFAULT_RESPONSE_PORT, short = "o", long = "output-port")]
    output_port: u32,
    #[structopt(default_value = "localhost", short = "h", long = "host")]
    host: String,
    #[structopt(short = "r", long = "request")]
    request_file: Option<String>,
}

fn get_url(host: &String, port: u32) -> String {
    format!("{}:{}", host, port)
}

fn handle_client(mut request_stream: TcpStream, opt: Opt)
            -> std::io::Result<()> {

    let sample_request : String = 
        if let Some(sample_request) = opt.request_file {
                fs::read_to_string(sample_request).unwrap()
        } else {
        r#"
{
  "command": "RSHOT",
  "objectType": "eventJSON",
  "siteCode": "ds12", 
  "holeNumber": "hole3",  
  "archiveFilename": "Archive_211001_140321", 
  "archivePath": "./path/to/archive/file/"
}
"#.to_string()
        };

    let r_json = serde_json::from_str::<Value>(&sample_request[..]).unwrap();

    let json_text = serde_json::to_string(&r_json).unwrap();
    println!("request(pretty validated)={}", json_text);
    println!("request(orig)={}", sample_request);

    // Note we send the original request that has been validated
    // by this point in the code.

    let len_msg = format!("{:08x}", sample_request.len());
    let wresult = request_stream.write(len_msg.as_bytes());

    match wresult {
        Err(e) => {
            println!("error writing len_msg: {}", e);
        }
        _ => {}
    }

    let wresult = request_stream.write(sample_request.as_bytes());
    match wresult {
        Err(e) => {
            println!("error writing json_text: {}", e);
        }
        _ => {}
    }

    let mut response_stream =
        TcpStream::connect(get_url(&opt.host, opt.output_port)).unwrap();
    let mut buffer = String::new();
    response_stream.read_to_string(&mut buffer)?;
    println!("{}", buffer);

    Ok(())
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    println!("opt={:?}", opt);

    let url = get_url(&opt.host, opt.input_port);
    let request_stream =
        match TcpStream::connect(&url[..]) {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("cannot connect {} {}", url, err);
                std::process::exit(1);
            }
        };

    handle_client(request_stream, opt)?;
    Ok(())
}
