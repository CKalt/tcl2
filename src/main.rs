use std::io::prelude::*;
use std::net::TcpStream;
use serde_json::Value;
use structopt::StructOpt;
use std::fs;
use std::str;

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
    #[structopt(short = "p", long = "pretty-json")]
    pretty_json: bool,
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

    let rq_json = serde_json::from_str::<Value>(&sample_request[..]).unwrap();

    if opt.pretty_json {
        let rq_json_text = serde_json::to_string_pretty(&rq_json).unwrap();
        println!("request(pretty validated)={}", rq_json_text);
    }

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
            println!("error writing rq_json_text: {}", e);
        }
        _ => {}
    }

    let mut response_stream =
        match TcpStream::connect(get_url(&opt.host, opt.output_port)) {
            Ok(response) => response,
            Err(e) => {
                eprintln!("connect failed on output_port e = {:?}", e);
                std::process::exit(1);
            }
        };

    let mut read_count = 100;

    loop {
        let mut len_buf: [u8; 8] = [0; 8];
        response_stream.read_exact(&mut len_buf).unwrap();
        println!("8 bytes read = {:?}", len_buf);
        let len_str = str::from_utf8(&len_buf).unwrap();
        let bytes_to_read: usize
            = usize::from_str_radix(len_str.trim(), 16).unwrap();
        println!("converts to hex str={} or bytes_to_read={}", len_str, bytes_to_read);

        // read exactly `bytes_to_read` len and error if not 
        // valid json
        let mut response_buf = vec![0u8; bytes_to_read];
        response_stream.read_exact(&mut response_buf).unwrap();
        let response = str::from_utf8(&response_buf).unwrap();
        println!("{} bytes received=[{}]", bytes_to_read, response);

        let rsp_json =
            serde_json::from_str::<Value>(response).unwrap();

        if opt.pretty_json {
            let rsp_json_text = serde_json::to_string_pretty(&rsp_json).unwrap();
            println!("response(pretty validated)={}", rsp_json_text);
        }

        read_count -= 1;
        if read_count == 0 {
            break;
        }
    }

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
                eprintln!("connect failed on input port {} {}", url, err);
                std::process::exit(1);
            }
        };

    handle_client(request_stream, opt)?;
    Ok(())
}
