use crate::*;
use http_service::Body;
use log::{debug, error};
use std::{net::IpAddr, thread};
use tide::{
    http,
    http::{header, StatusCode},
    Context,
};

pub struct Httpd {
    app: tide::App<Binary>,
    ip: IpAddr,
    port: u16,
    bin: Binary,
}

impl Httpd {
    pub fn new(ip: &IpAddr, port: u16, bin: &Binary) -> Self {
        let mut app = tide::App::with_state(bin.clone());

        let endpoint = format!("/{}", bin.basename());
        app.at(&endpoint).get(|cx: Context<Binary>| {
            async move {
                println!("serve requested binary: {}", cx.state());
                debug!("request headers: {:#?}", cx.headers());
                match cx.state().slurp() {
                    Ok(content) => {
                        let resp = if let Some(range_header) = cx.headers().get("Range") {
                            match Httpd::parse_range_header(range_header) {
                                Ok((from, to)) => {
                                    let content_range =
                                        format!("bytes {}-{}/{}", from, to, content.len());
                                    let chunk = content[from..=to].to_vec();

                                    http::Response::builder()
                                        .status(StatusCode::PARTIAL_CONTENT)
                                        .header(header::CONTENT_TYPE, "application/octet-stream")
                                        .header(header::CONTENT_LENGTH, chunk.len())
                                        .header(header::CONTENT_RANGE, content_range)
                                        .body(Body::from(chunk))
                                        .unwrap()
                                }
                                Err(err) => {
                                    error!("{}", err);
                                    return Httpd::error_response(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        err.to_string(),
                                    );
                                }
                            }
                        } else {
                            http::Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/octet-stream")
                                .header(header::CONTENT_LENGTH, content.len())
                                .body(Body::from(content))
                                .unwrap()
                        };

                        debug!("response: {:#?}", resp);
                        resp
                    }
                    Err(err) => {
                        error!("unable to read binary: {} - {}", cx.state(), err);
                        Httpd::error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                    }
                }
            }
        });

        Httpd {
            app,
            ip: *ip,
            port,
            bin: bin.clone(),
        }
    }

    pub fn start(self) -> (String, thread::JoinHandle<()>) {
        let bin_endpoint = format!("http://{}:{}/{}", &self.ip, self.port, self.bin.basename());
        let hndl = thread::spawn(|| {
            let addr = format!("{}:{}", self.ip, self.port);
            debug!("startup web-server to serve image at {}", &addr);
            self.app.serve(&addr).expect("unable to start web-server");
        });
        (bin_endpoint, hndl)
    }

    fn parse_range_header(value: &http::HeaderValue) -> Result<(usize, usize)> {
        fn split_at_char(s: String, c: char) -> Result<(String, String)> {
            let mut iter = s.splitn(2, c);
            match (iter.next(), iter.next()) {
                (Some(a), Some(b)) => Ok((a.to_string(), b.to_string())),
                _ => Err(Error::ParserError {
                    msg: format!("unable to split: '{}'", s),
                }),
            }
        }

        let (type_, range) = split_at_char(value.to_str().unwrap().to_string(), '=')?;
        if type_ != "bytes" {
            return Err(Error::ParserError {
                msg: format!("unexpected range type: '{}'", type_),
            });
        }

        let (from, to) = split_at_char(range, '-')?;
        Ok((from.parse()?, to.parse()?))
    }

    fn error_response<S>(code: StatusCode, body: S) -> http::Response<Body>
    where
        S: Into<Body>,
    {
        http::Response::builder()
            .status(code)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(body.into())
            .expect("unable to build err response")
    }
}
