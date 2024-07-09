use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        thread::spawn(|| {
            let buff: &mut [u8] = &mut [0; 100];
            match stream {
                Ok(mut _stream) => {
                    read_to_buffer(&_stream, buff);
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        });
    }
}

fn read_to_buffer(mut stream: &TcpStream, buf: &mut [u8]) {
    let mut request: Vec<u8> = Vec::new();
    loop {
        if let Ok(size) = stream.read(buf) {
            buf.iter().take(size).for_each(|b| request.push(*b));

            if size < buf.len() {
                break;
            }
        } else {
            break;
        }
    }

    let request = std::str::from_utf8(&request).expect("parsing");

    let request = request.split_once("\r\n");
    let (request, header, body) = match request {
        Some((request_line, header_line)) => {
            let (header, body) = get_body(header_line);
            (request_line, header, body)
        }
        None => ("", "", ""),
    };

    let request_line = request
        .split(" ")
        .map(|i| i.to_string())
        .collect::<Vec<String>>();

    let response = match &request_line[1] {
        file_fetch if file_fetch.starts_with("/files/") => file_fetch_handler(file_fetch),
        user_agent if user_agent == "/user-agent" => user_agent_handler(&header),
        echo if echo.starts_with("/echo/") => {
            let data = echo.trim_start_matches("/echo/");
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                data.len(),
                data,
            )
        }
        success if success == "/" => {
            format!("HTTP/1.1 200 OK\r\n\r\n")
        }
        _ => {
            format!("HTTP/1.1 404 Not Found\r\n\r\n")
        }
    };

    let _ = stream.write_all(response.as_bytes());
}

fn file_fetch_handler(file_fetch: &str) -> String {
    // let prefix = "/tmp/";
    let prefix = std::env::args().nth(2).expect("test");
    let data = file_fetch.trim_start_matches("/files/");
    let file_path = PathBuf::from(format!("{}{}", prefix, data));
    let canonical_path_result = fs::canonicalize(file_path);

    let canonical_path = match canonical_path_result {
        Err(_) => return not_found(),
        Ok(path) => {
            // if !path.starts_with(prefix) {
            //     return not_found();
            // }
            path
        }
    };

    let mut buf: Vec<u8> = Vec::new();
    let file = fs::File::open(canonical_path);

    let _ = match file {
        Ok(mut f) => f.read_to_end(&mut buf),
        _ => return not_found(),
    };

    let body = match std::str::from_utf8(&buf) {
        Ok(b) => b,
        _ => return not_found(),
    };

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
        buf.len(),
        body
    )
}

fn user_agent_handler(header: &str) -> String {
    let header_list = header.split("\r\n").collect();
    let user_agent = get_user_agent(&header_list);

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        user_agent.len(),
        user_agent
    )
}

fn get_user_agent(header_line: &Vec<&str>) -> String {
    let user_agent_filter = header_line
        .into_iter()
        .find(|line| line.starts_with("User-Agent:"));

    match user_agent_filter {
        Some(user_agent_line) => user_agent_line.split(":").collect::<Vec<&str>>()[1]
            .trim()
            .to_string(),
        None => "".to_string(),
    }
}

fn get_body<'a>(r: &'a str) -> (&str, &str) {
    let r = r.split_once("\r\n\r\n");

    match r {
        Some((header, body)) => (header, body),
        None => ("", ""),
    }
}

fn not_found() -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".into()
}
