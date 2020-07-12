use std::net::{TcpStream};
use std::io::{Write, BufRead};
use std::io::BufReader;

use ham_rs::mode::Mode;
use ham_rs::{FrequencyResponse,CommandError,CommandResponse,ModeResponse};

pub fn get_frequency(stream:&mut TcpStream) -> Result<FrequencyResponse,CommandError> {
    stream.write(format!("f\n").as_bytes())?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    reader.read_line(&mut line)?;
    let freq = line.trim();
    let freq = freq.parse::<i32>()?;

    // TODO: This used to work now it does not ¯\_(ツ)_/¯ 
    /*let mut line = String::new();
    reader.read_line(&mut line)?;
    let vfo = line.trim();

    println!("vfo: {}", vfo);*/
    let vfo = "";

    Ok(FrequencyResponse{frequency:freq,vfo: vfo.to_string()})
}

pub fn change_frequency(stream:&mut TcpStream, freq: i32) -> Result<CommandResponse,CommandError> {
    stream.write(format!("F {}\n", freq).as_bytes())?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    match reader.read_line(&mut line) {
        Ok(_) if line.trim() == "RPRT 0" => Ok(CommandResponse::Status(0)),
        Ok(_) => Err(CommandError::UnexpectedResponse(line)),
        Err(e) => Err(CommandError::IoError(e)),
    }
}

pub fn get_mode(stream:&mut TcpStream) -> Result<ModeResponse,CommandError> {
    stream.write(format!("m\n").as_bytes())?;
    let mut reader = BufReader::new(stream);

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mode =
        match line.trim() {
            "DigiU" => Ok(Mode::DigiU),
            "DigiL" => Ok(Mode::DigiL),
            "USB" => Ok(Mode::USB),
            "LSB" => Ok(Mode::LSB),
            "FT8" => Ok(Mode::FT8),
            "FT4" => Ok(Mode::FT4),
            "JT9" => Ok(Mode::JT9),
            "AM" => Ok(Mode::AM),
            "FM" => Ok(Mode::FM),
            "NFM" => Ok(Mode::NFM),
            "WSPR" => Ok(Mode::WSPR),
            "PSK" => Ok(Mode::PSK),
            "Multipsk" => Ok(Mode::Multipsk),
            "Sig" => Ok(Mode::Sig),
            "Hell" => Ok(Mode::Hell),
            "CW" => Ok(Mode::CW),
            mode => Err(CommandError::UnexpectedResponse(mode.to_string()))
        };

    match mode {
        Ok(mode) => {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let width = line.trim();
            let width = width.parse::<i32>()?;

            Ok(ModeResponse{ mode: mode, width: width})
        },
        Err(e) => Err(e)
    }
}

pub fn change_mode(stream:&mut TcpStream, mode: &Mode) -> Result<CommandResponse,CommandError> {
    let mode_string =
        match mode {
            Mode::DigiU => "DigiU",
            Mode::DigiL => "DigiL",
            Mode::USB => "USB",
            Mode::LSB => "LSB",
            Mode::FT8 => "FT8",
            Mode::FT4 => "FT4",
            Mode::JT9 => "JT9",
            Mode::AM => "AM",
            Mode::FM => "FM",
            Mode::NFM => "NFM",
            Mode::WSPR => "WSPR",
            Mode::PSK => "PSK",
            Mode::Multipsk => "Multipsk",
            Mode::Sig => "Sig",
            Mode::Hell => "Hell",
            Mode::CW => "CW",
            Mode::Other(_) => "USB"
        };
    stream.write(format!("M {} 0\n", mode_string).as_bytes())?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    match reader.read_line(&mut line) {
        Ok(_) if line.trim() == "RPRT 0" => Ok(CommandResponse::Status(0)),
        Ok(_) => Err(CommandError::UnexpectedResponse(line)),
        Err(e) => Err(CommandError::IoError(e)),
    }
}
