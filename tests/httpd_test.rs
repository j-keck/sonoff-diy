use reqwest::header::RANGE;
use sonoff_diy::*;
use std::net::{IpAddr, Ipv4Addr};

const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[test]
fn download_range() {
    env_logger::init();

    let bin = Binary::new("shell.nix").unwrap();
    let bin_content = bin.slurp().unwrap();
    let bin_len = bin_content.len();

    let httpd = Httpd::new(&LOCALHOST, 9876, &bin).unwrap();
    let (endpoint, _) = httpd.start();

    let mut resp_content = Vec::new();
    let client = reqwest::Client::new();

    // fetch the first 100 bytes
    let mut resp = client
        .get(&endpoint)
        .header(RANGE, "bytes=0-99")
        .send()
        .unwrap();
    assert_eq!(resp.headers().get("content-length").unwrap(), "100");
    resp.copy_to(&mut resp_content).unwrap();

    // fetch the rest
    let mut resp = client
        .get(&endpoint)
        .header(RANGE, format!("bytes=100-{}", bin_len - 1))
        .send()
        .unwrap();
    assert_eq!(
        resp.headers().get("content-length").unwrap(),
        &(bin_len - 100).to_string()
    );
    resp.copy_to(&mut resp_content).unwrap();

    // compare the response content with the original file content
    assert_eq!(bin_content, resp_content);
}
