use sonoff_diy::*;
use log::{error, info};
use serde_json::to_string_pretty;
use std::{error::Error, io, io::Read};

#[paw::main]
fn main(args: Args) {
    init_logger(&args);

    // the binary to flash
    let bin = Binary::new(args.bin).unwrap();

    let httpd = Httpd::new(args.httpd_port, &bin);
    let mut scanner = Scanner::new(args.service_name);
    if let Err(err) = scan_loop(&mut scanner, httpd, &bin) {
        eprintln!("{}", err);
    }
}

fn scan_loop(scanner: &mut Scanner, httpd: Httpd, bin: &Binary) -> Result<(), Box<dyn Error>> {
    let device = scanner.scan()?;

    info!("new device found: {:?}", device);
    match device.info() {
        Ok(info) => {
            info!("  infos: {:?}", to_string_pretty(&info));
            println!("\n\nflash it? [y/N]");
            if user_resp_was_y() {
                let httpd_ip = netutils::matching_host_ip_for(&device.ip);

                // start the embedded web-server to serve the firmware
                let endpoint = httpd.start(&httpd_ip);

                device.unlock();
                device.flash(endpoint, bin);
                Ok(())
            } else {
                scan_loop(scanner, httpd, bin)
            }
        }
        Err(err) => {
            error!("unable to fetch device infos: {}", err);
            scan_loop(scanner, httpd, bin)
        },
    }
}

fn user_resp_was_y() -> bool {
    if let Some(c) = io::stdin()
        .bytes()
        .next()
        .and_then(|r| r.ok())
        .map(|b| b as char)
    {
        c == 'y'
    } else {
        false
    }
}

fn init_logger(args: &Args) {
    let filter = {
        let level = if args.verbose { "debug" } else { "info" };
        format!("{}={}", env!("CARGO_PKG_NAME"), level)
    };

    env_logger::from_env(env_logger::Env::default().default_filter_or(filter)).init();
}
