#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate shellexpand;

use std::path::Path;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;

use clap::clap_app;
use clap::AppSettings;
use please_clap::clap_dispatch;

#[derive(Debug, Serialize, Deserialize)]
enum ReceiverMode {
    DigiU,
    DigiL,
    USB,
    LSB,
    FT8,
    FT4,
    JT9,
    AM,
    FM,
    NFM,
    WSPR,
    PSK,
    Multipsk,
    Sig,
    Hell,
    CW
}

#[derive(Debug, Serialize, Deserialize)]
struct RigCtl {
    freq: i32,
    mode: ReceiverMode
}

type ReceiverConfig = BTreeMap<String,BTreeMap<String,RigCtl>>;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    receivers: ReceiverConfig
}

fn change_frequency(stream:&mut TcpStream, freq: i32) -> Result<String,std::io::Error> {
    let msg = format!("F {}\n", freq);
    stream.write(msg.as_bytes()).unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    match reader.read_line(&mut line) {
        Ok(_) => Ok(line),
        Err(e) => Err(e),
    }
}
fn change_mode(stream:&mut TcpStream, mode: &ReceiverMode) -> Result<String,std::io::Error> {
    let mode_string =
        match mode {
            ReceiverMode::DigiU => "DigiU",
            ReceiverMode::DigiL => "DigiL",
            ReceiverMode::USB => "USB",
            ReceiverMode::LSB => "LSB",
            ReceiverMode::FT8 => "FT8",
            ReceiverMode::FT4 => "FT4",
            ReceiverMode::JT9 => "JT9",
            ReceiverMode::AM => "AM",
            ReceiverMode::FM => "FM",
            ReceiverMode::NFM => "NFM",
            ReceiverMode::WSPR => "WSPR",
            ReceiverMode::PSK => "PSK",
            ReceiverMode::Multipsk => "Multipsk",
            ReceiverMode::Sig => "Sig",
            ReceiverMode::Hell => "Hell",
            ReceiverMode::CW => "CW"
        };

    let msg = format!("M {} 0\n", mode_string);
    stream.write(msg.as_bytes()).unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    match reader.read_line(&mut line) {
        Ok(_) => Ok(line),
        Err(e) => Err(e),
    }
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

        (@subcommand run =>
            (about: "run profile")
            (@arg profile_name: +required "Run profile for each receiver")));

    app = app.setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();

    clap_dispatch!(matches; {
        run(_, profile_name as profile_name) => {
            for (connection_string, profiles) in &config.receivers {
                match TcpStream::connect(&connection_string) {
                    Ok(mut stream) => {
                        match profiles.get(&profile_name.to_string()) {
                            Some(profile) => {
                                change_frequency(&mut stream, profile.freq).unwrap();
                                change_mode(&mut stream, &profile.mode).unwrap();
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
