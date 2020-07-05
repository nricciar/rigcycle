#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate shellexpand;
extern crate regex;
extern crate ws;
extern crate portaudio;
extern crate hound;

use std::path::Path;
use std::net::{TcpStream};
use std::io::{Read};
use std::fs::File;
use clap::clap_app;
use clap::AppSettings;
use please_clap::clap_dispatch;
use regex::Regex;
use ws::{listen};
use std::collections::BTreeMap;
use ham_rs::mode::Mode;

mod rig;
mod server;

use server::{Server};

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

fn main() {
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

        (@subcommand ws =>
            (about: "websocket service")
            (@arg port: +required "Port to listen on"))
        (@subcommand run =>
            (about: "run profile")
            (@arg profile_name: +required "Run profile for each receiver")));

    app = app.setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();

    clap_dispatch!(matches; {
        ws(_, port as port) => {
            listen(format!("127.0.0.1:{}",port), |out| Server { out: out, config: config.clone() }).unwrap();
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
}
