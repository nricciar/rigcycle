#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate shellexpand;
extern crate ws;
extern crate portaudio;
extern crate hound;
extern crate tokio;
extern crate futures;
extern crate websocket;

use std::path::Path;
use std::net::{TcpStream};
use std::io::{Read};
use std::fs::File;
use clap::clap_app;
use clap::AppSettings;
use please_clap::clap_dispatch;
use ws::{listen};
use std::collections::BTreeMap;
use ham_rs::mode::Mode;
use native_tls::{Identity};

mod rig;
mod server;
mod proxy;

use server::{Server};
use proxy::{ProxyService};

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct RigCtl {
    pub freq: i32,
    pub mode: Mode
}

pub type ReceiverConfig = BTreeMap<String,BTreeMap<String,RigCtl>>;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Config {
    pub receivers: ReceiverConfig
}
impl Config {
    pub fn receivers(&self) -> Vec<ham_rs::Receiver> {
        let mut count = 0;
        self.receivers.iter().map(|(connection_string, _)| {
            let mut stream = TcpStream::connect(&connection_string).unwrap();
            let freq = crate::rig::get_frequency(&mut stream).unwrap();
            let mode = crate::rig::get_mode(&mut stream).unwrap();
            let ret = ham_rs::Receiver { id: count, frequency: freq.frequency, mode: mode.mode };
            count += 1;
            ret
        }).collect()
    }

    pub fn connection_strings(&self) -> Vec<String> {
        self.receivers.iter().map(|(connection_string, _)| {
            connection_string.to_string()
        }).collect()
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let conf_dir_full = shellexpand::tilde("~/.rigcycle").into_owned();
    let conf_dir = Path::new(&conf_dir_full);
    let config_file = conf_dir.join("config.yaml");

    let mut file = File::open(&config_file).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let config : Config = serde_yaml::from_str(&data).unwrap();

    let mut app = clap_app!(encode =>
        (version: "0.1.0")
        (about: "rigcycle")

        (@subcommand proxy =>
            (about: "proxy websocket service")
            (@arg client_cert: +required "LOTW Client certificate")
            (@arg client_cert_pass: --password +takes_value "Client certificate password (default: prompt)"))
        (@subcommand ws =>
            (about: "websocket service")
            (@arg port: +required "Port to listen on"))
        (@subcommand run =>
            (about: "run profile")
            (@arg profile_name: +required "Run profile for each receiver")));

    app = app.setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();

    clap_dispatch!(matches; {
        proxy(proxy_matches, client_cert as client_cert) => {
            let mut f = File::open(&client_cert).expect("no file found");
            let metadata = std::fs::metadata(&client_cert).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read(&mut buffer).expect("buffer overflow");
            let pass = 
                match proxy_matches.value_of("client_cert_pass") {
                    Some(pass) => pass.to_string(),
                    None => rpassword::read_password_from_tty(Some("Password: ")).unwrap(),
                };
            let cert = Identity::from_pkcs12(&buffer, &pass).unwrap();

            let proxy = ProxyService::new(config.clone(), cert);
            proxy.run();
        },
        ws(_, port as port) => {
            listen(format!("127.0.0.1:{}",port), |out| {
                Server { out: out, config: config.clone() }
            }).unwrap();
        },
        run(_, profile_name as profile_name) => {
            for (connection_string, profiles) in &config.receivers {
                match TcpStream::connect(&connection_string) {
                    Ok(mut stream) => {
                        match profiles.get(&profile_name.to_string()) {
                            Some(profile) => {
                                rig::change_frequency(&mut stream, profile.freq).unwrap();
                                rig::change_mode(&mut stream, &profile.mode).unwrap();
                                println!("Updated {} to {} / {:?}", connection_string, profile.freq, profile.mode);
                            },
                            None => {
                                println!("Skipping {}. No {} profile", connection_string, profile_name);
                            }
                        }
                    },
                    Err(e) => {
                        println!("Failed to connect to {}: {}", connection_string, e);
                    }
                }
            }
        }
    });

    Ok(())
}
