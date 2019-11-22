use crate::Binary;
use log::{error, info};
use std::{thread, net::IpAddr};

use http_service::Body;
use tide::{
    http,
    http::{header, StatusCode},
    Context,
};

pub struct Httpd {
    app: tide::App<Binary>,
    port: u16,
    bin: Binary,
}

impl Httpd {
    pub fn new(port: u16, bin: &Binary) -> Self {
        let mut app = tide::App::with_state(bin.clone());

        let endpoint = format!("/{}", bin.basename());
        app.at(&endpoint).get(|cx: Context<Binary>| {
            async move {
                match cx.state().read() {
                    Ok(content) => {
                        info!("serve binary: {}", cx.state());
                        http::Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/octet-stream")
                            .header(header::CONTENT_LENGTH, content.len())
                            .body(Body::from(content))
                            .expect("unable to build response")
                    }
                    Err(err) => {
                        error!("unable to read binary: {} - {}", cx.state(), err);
                        http::Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(header::CONTENT_TYPE, "text/plain")
                            .body(Body::from(err.to_string()))
                            .expect("unable to build err response")
                    }
                }
            }
        });

        Httpd { app, port, bin: bin.clone() }
    }


    pub fn start(self, ip: &IpAddr) -> String {
        let endpoint = format!("http://{}:{}/{}", &ip, self.port, self.bin.basename());

        let ip = ip.clone();
        thread::spawn(move || {
            let addr = format!("{}:{}", ip, self.port);
            info!("startup web-server to serve image at {}", &addr);
            self.app.serve(&addr).expect("unable to start web-server");
        });

        endpoint
    }

}
