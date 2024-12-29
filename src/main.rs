use std::{
    fs, io::{BufRead, BufReader, BufWriter, Write as _}, net::{TcpListener, TcpStream}, str::from_utf8, time::Duration
};

use http::{header, HeaderMap, HeaderName, HeaderValue, Request, Response};

fn handle_connection(stream: TcpStream) {
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    // let mut buffer: Vec<u8> = Vec::new();

    let request = parse_request(stream);
    let base_path = "files".to_string();
    let request_path = match request.uri().path() {
        "/" => "/index.html",
        str => str,
    };
    let full_path = base_path + request_path;

    // Read requested file
    let file_result = fs::read(&full_path);
    // Determine if file exists
    if file_result.is_err() {
        let _ = writer.write(b"HTTP/1.1 404");
    } else {
	let file_content = file_result.unwrap();
	let response = response(file_content);
	// Print utf8 responses in console
	match from_utf8(&response) {
	    Ok(v) => println!("{}", v),
	    Err(v) => ()
	}
        let _ = writer.write(&response).unwrap();
    }
}

fn parse_request(stream: TcpStream) -> http::Request<String> {
    let reader = BufReader::new(stream.try_clone().unwrap());
    let mut request = Request::new(String::new());

    for line in reader.lines().map(|l| l.unwrap().trim_end().to_owned()) {
        if line.find(':').is_some() {
            let pair: Vec<&str> = line.split(':').collect();
            let header_name =
                HeaderName::from_lowercase(&pair[0].to_lowercase().into_bytes()).unwrap();
            let value = HeaderValue::from_str(pair[1]).unwrap();
            request.headers_mut().insert(header_name, value);
            if valid_headers(request.headers()) {
                break;
            }
        } else if line.len() != 0 {
            let path = line.split_whitespace().nth(1).unwrap();
            *request.uri_mut() = path.parse().unwrap();
        }
    }
    request
}

fn response(content: Vec<u8>) -> Vec<u8> {
    let mut response = Response::builder()
        .header("Content-Type", "text/html")
        .status(200)
        .body(content)
        .unwrap();
    let mut headers: Vec<String> = vec![];
    headers.append(&mut vec![format!("HTTP/1.1 {}", response.status())]);
    // TODO: Status code
    for (name, value) in response.headers() {
	let header_line = format!("{}: {}", name.as_str(), value.to_str().unwrap());
	headers.append(&mut vec![header_line]);
    }
    let mut final_response = headers.join("\n").into_bytes();
    final_response.append(&mut (b"\n\n").to_vec());
    final_response.append(&mut response.body().to_vec());
    final_response
}

fn valid_headers(headers: &HeaderMap<HeaderValue>) -> bool {
    let required_headers = [header::ACCEPT, header::ACCEPT_LANGUAGE, header::DNT];
    let mut valid = true;
    for key in required_headers {
        if !headers.contains_key(key) {
            valid = false;
        }
    }
    valid
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let socket_addr = &listener.local_addr().unwrap();
    println!("{socket_addr}");

    for stream in listener.incoming() {
        let m_stream = stream;
        match m_stream {
            Ok(m_stream) => {
                m_stream.set_nonblocking(false).unwrap();
                let _ = m_stream.set_read_timeout(Some(Duration::new(5, 0)));
                handle_connection(m_stream);
            }
            Err(e) => {
                println!("{e}");
            }
        }
    }
}
