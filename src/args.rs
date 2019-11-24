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

        #[structopt(long)]
        /// firmware binary to flash
        bin: String,

        #[structopt(long, default_value = "8989")]
        httpd_port: u16,
    },
}

#[derive(StructOpt, Debug)]
pub enum SwitchState {
    On,
    Off,
}
