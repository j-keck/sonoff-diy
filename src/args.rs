use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(long)]
    /// firmware binary to flash
    pub bin: String,

    #[structopt(long, default_value = "8989")]
    pub httpd_port: u16,

    #[structopt(short, long)]
    pub verbose: bool,

    #[structopt(long, default_value = "_ewelink._tcp")]
    pub service_name: String,
}
