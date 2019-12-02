use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(short, long)]
    /// enable debug logging
    pub debug: bool,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Scan devices
    Scan {
        #[structopt(long, default_value = "_ewelink._tcp")]
        service_name: String,
    },

    /// List found devices
    List,

    /// fetch device infos
    Info {
        #[structopt(long, short = "id")]
        device_id: String,
    },

    /// set wifi credentials in the sonoff device
    Wifi {
        #[structopt(long, short = "id")]
        device_id: String,

        #[structopt(long)]
        essid: String,

        #[structopt(long)]
        pwd: String,
    },

    /// switch on / off
    Switch {
        #[structopt(long, short = "id")]
        device_id: String,

        #[structopt(subcommand)]
        state: SwitchState,
    },

    /// unlock the sonoff device to flash it ota
    Unlock {
        #[structopt(long, short = "id")]
        device_id: String,
    },

    /// flash the given firmware (--bin) ota
    Flash {
        #[structopt(long, short = "id")]
        device_id: String,

        #[structopt(
            long,
            conflicts_with = "external_httpd_url",
            conflicts_with = "bin_sha256sum"
        )]
        /// firmware binary to flash
        bin: Option<String>,

        #[structopt(
            long,
            default_value = "8989",
            conflicts_with = "external_httpd_url",
            conflicts_with = "bin_sha256sum"
        )]
        httpd_port: u16,

        #[structopt(long, conflicts_with = "bin", conflicts_with = "httpd_port")]
        /// use external web-server with the given url (http://<IP>:<PORT>/path/sonoff.bin)
        external_httpd_url: Option<String>,

        #[structopt(long, conflicts_with = "bin", conflicts_with = "httpd_port")]
        /// when using the extenal web-server, we need the sha256 sum of the binary on the server
        bin_sha256sum: Option<String>,
    },
}

#[derive(StructOpt, Debug)]
pub enum SwitchState {
    On,
    Off,
}
