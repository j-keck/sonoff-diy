use sonoff_diy::*;

#[paw::main]
fn main(args: Args) {
    init_logger(&args);
    if let Err(err) = run(args) {
        eprintln!("Error: {}", err);
        err.print_backtrace();
    }
}

fn run(args: Args) -> Result<()> {
    let mut device_cache = DeviceCache::load().unwrap_or_default();

    Ok(match args.cmd {
        Command::Scan { service_name } => {
            println!("scan for devices in the current network");
            let mut scanner = Scanner::new(service_name);
            scanner.scan_loop(move |device| {
                let device = device?;
                println!("{}", device);
                device_cache.add(&device)
            })?
        }
        Command::List => {
            for device in device_cache.devices() {
                println!("{}", device);
            }
        }
        Command::Info { device_id } => {
            let device = device_cache.lookup(&device_id)?;
            println!("{}", device.info()?)
        }
        Command::Wifi {
            device_id,
            essid,
            pwd,
        } => {
            let device = device_cache.lookup(&device_id)?;
            println!("{}", device.wifi(essid, pwd)?)
        }
        Command::Switch { device_id, state } => {
            let device = device_cache.lookup(&device_id)?;
            println!("{}", device.switch(state)?)
        }
        Command::Unlock { device_id } => {
            let device = device_cache.lookup(&device_id)?;
            println!("{}", device.unlock()?)
        }
        Command::Flash {
            device_id,
            bin,
            httpd_port,
        } => {
            let bin = Binary::new(bin)?;
            let device = device_cache.lookup(&device_id)?;

            let httpd_ip = netutils::matching_host_ip_for(&device.ip)?;
            println!(
                "startup the embedded web-server at {} to serve the binary",
                httpd_ip
            );
            let httpd = Httpd::new(&httpd_ip, httpd_port, &bin)?;
            let (bin_endpoint, hndl) = httpd.start();
            println!("{}", device.flash(bin_endpoint, &bin)?);
            println!("hit <CTRL-C> to shudown the embedded web-server");
            hndl.join().unwrap();
        }
    })
}

fn init_logger(args: &Args) {
    let filter = {
        let level = if args.debug { "debug" } else { "info" };
        let name = env!("CARGO_PKG_NAME").replace("-", "_");
        format!("{}={}", name, level)
    };

    env_logger::from_env(env_logger::Env::default().default_filter_or(filter)).init();
}
