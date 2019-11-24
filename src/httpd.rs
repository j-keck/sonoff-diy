use crate::*;
use log::debug;
use std::{
    cmp,
    collections::HashMap,
    io::{BufWriter, Read, Write},
    net::{IpAddr, TcpListener, TcpStream},
    thread,
};

pub struct Httpd {
    ip: IpAddr,
    port: u16,
    bin: Binary,
    bin_content: Vec<u8>,
}

type Method = String;
type Path = String;
type Headers = HashMap<String, String>;
type Body = String;

impl Httpd {
    pub fn new(ip: &IpAddr, port: u16, bin: &Binary) -> Result<Self> {
        let ip = *ip;
        let bin = bin.clone();
        let bin_content = bin.slurp()?;
        Ok(Httpd {
            ip,
            port,
            bin,
            bin_content,
        })
    }

    pub fn start(self) -> (String, thread::JoinHandle<()>) {
        let bin_endpoint = format!("http://{}:{}/{}", &self.ip, self.port, self.bin.basename());
        let hndl = thread::spawn(move || {
            let listener = TcpListener::bind((self.ip, self.port)).unwrap();
            for stream in listener.incoming() {
                let mut stream = stream.expect("unable to start server");
                self.handle_request(&mut stream);
            }
        });
        (bin_endpoint, hndl)
    }

    fn handle_request(&self, stream: &mut TcpStream) {
        let mut buf = [0_u8; 4096];
        let n = stream.read(&mut buf).expect("unable to read from stream");
        if n == 0 {
            debug!("empty request");
            return;
        }
        match Httpd::parse_request(&buf[..n]) {
            Ok((method, path, headers, _))
                if method == "GET" && path.contains(&self.bin.basename()) =>
            {
                match self.handle_bin_download(stream, headers) {
                    Ok(_) => self.handle_request(stream),
                    Err(err) => eprintln!("Unable to serve the binary: {}", err),
                }
            }
            Ok((method, _, _, body)) if method == "POST" => {
                println!("upload done - resonse: {}", body)
            }
            Ok(t) => eprintln!("unexpected request: {:#?}", t),
            Err(err) => eprintln!("{}", err),
        };
    }

    fn parse_request(raw: &[u8]) -> Result<(Method, Path, Headers, Body)> {
        let raw = String::from_utf8_lossy(raw);
        debug!("parse request: {}", raw);
        match raw.splitn(2, "\r\n\r\n").collect::<Vec<_>>().as_slice() {
            [header, body] => match header
                .splitn(4, |c: char| c.is_whitespace())
                .collect::<Vec<_>>()
                .as_slice()
            {
                [method, path, _, raw_headers] => {
                    let headers = Httpd::parse_headers(raw_headers);
                    Ok((
                        method.to_string(),
                        path.to_string(),
                        headers,
                        body.to_string(),
                    ))
                }
                _ => Err(Error::InvalidRequest {
                    msg: format!("unexpected request: '{}'", raw),
                }),
            },
            _ => Err(Error::InvalidRequest {
                msg: format!("header / body separator in '{}' not found", raw),
            }),
        }
    }

    fn parse_headers(s: &str) -> Headers {
        let mut headers = HashMap::new();
        for line in s.lines() {
            let mut iter = line.splitn(2, ": ");
            match (iter.next(), iter.next()) {
                (Some(name), Some(value)) => {
                    let name = name.to_lowercase();
                    headers.insert(name, value.into());
                }
                _ => debug!("ignore invalid header: {}", line),
            }
        }
        headers
    }

    fn handle_bin_download(
        &self,
        stream: &mut TcpStream,
        headers: HashMap<String, String>,
    ) -> Result<()> {
        // parse the range header
        let range_header = headers.get("range").ok_or(Error::InvalidRequest {
            msg: "Range header not found".into(),
        })?;
        let len = self.bin_content.len();
        let (from, to) = Httpd::parse_range_header(&range_header)?;
        println!(
            "{:5.1}% - serve chunk from: {}, to: {}",
            100.0 / len as f32 * (to + 1) as f32,
            from,
            to
        );

        // extract the requested chunk
        let to = cmp::min(to, len - 1);
        let chunk = &self.bin_content[from..=to];

        let mut b = BufWriter::new(stream);
        b.write_all(b"HTTP/1.1 206 Partial Content\r\n")?;
        b.write_all(b"Content-Type: application/octet-stream\r\n")?;
        let content_length = format!("Content-Length: {}\r\n", chunk.len());
        b.write_all(content_length.as_bytes())?;
        let content_range = format!(
            "Content-Range: bytes {}-{}/{}\r\n",
            from,
            to,
            self.bin_content.len()
        );
        debug!("respond with content-range: {}", content_range);
        b.write_all(content_range.as_bytes())?;
        b.write_all(b"\r\n")?;
        b.write_all(chunk)?;
        b.flush()?;
        Ok(())
    }

    fn parse_range_header(value: &str) -> Result<(usize, usize)> {
        fn split_at_char(s: &str, c: char) -> Result<(String, String)> {
            let mut iter = s.splitn(2, c);
            match (iter.next(), iter.next()) {
                (Some(a), Some(b)) => Ok((a.to_string(), b.to_string())),
                _ => Err(Error::ParserError {
                    msg: format!("unable to split: '{}'", s),
                }),
            }
        }

        let (type_, range) = split_at_char(value, '=')?;
        if type_ != "bytes" {
            return Err(Error::ParserError {
                msg: format!("unexpected range type: '{}'", type_),
            });
        }

        let (from, to) = split_at_char(&range, '-')?;
        Ok((from.parse()?, to.parse()?))
    }
}
