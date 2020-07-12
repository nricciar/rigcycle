use websocket::{ClientBuilder,OwnedMessage};
use native_tls::{TlsConnector};
use ham_rs::{Command,CommandMessage,CommandResponseBody,CommandResponse};
use std::net::{TcpStream};

pub static PROXY_SERVICE_LOCATION: &str = "wss://rig.kk4wjs.com/ws";

pub struct ProxyService {
    config: crate::Config,
    cert: native_tls::Identity,
}

impl ProxyService {
    pub fn new(config: crate::Config, identity: native_tls::Identity) -> ProxyService {
        ProxyService {
            config: config,
            cert: identity,
        }
    }

    pub fn run(&self) {
        let mut tls_conn_builder = TlsConnector::builder();
        tls_conn_builder.identity(self.cert.clone());
        let tls_conn = tls_conn_builder.build().unwrap();
        let mut client = ClientBuilder::new(PROXY_SERVICE_LOCATION).unwrap()
            .connect_secure(Some(tls_conn))
            .unwrap();

        println!("Connected to {}", PROXY_SERVICE_LOCATION);

        loop {
            let msg = client.recv_message().unwrap();
            match msg {
                OwnedMessage::Text(cmd) => {
                    match self.handle_command(&cmd) {
                        Some(response) => {
                            let resp = serde_json::to_string(&response).unwrap();
                            println!(">> {}", resp);
                            client.send_message(&OwnedMessage::Text(resp)).unwrap();
                        }
                        None => ()
                    }
                },
                _ => {
                    eprintln!("ERROR! Unexpected message from server: {:?}", msg);
                }
            }
            
        }
    }

    fn handle_command(&self, msg: &str) -> Option<CommandResponseBody> {
        println!("<< {}", msg);
        let cmd : CommandMessage = serde_json::from_str(msg).unwrap();
        match (cmd.receiver, cmd.cmd) {
            (None, Command::getReceivers) => {
                let receivers = self.receivers();
                Some(CommandResponseBody { response: CommandResponse::ReceiverList(receivers), receiver: None })
            },
            (Some(receiver_id), Command::getFrequency) => {
                let connection_string = &self.connection_strings()[receiver_id as usize];
                match TcpStream::connect(connection_string) {
                    Ok(mut stream) => {
                        let freq = crate::rig::get_frequency(&mut stream).unwrap();
                        Some(CommandResponseBody { response: CommandResponse::FrequencyResponse(freq), receiver: Some(receiver_id) })
                    },
                    Err(e) => {
                        eprintln!("ERROR! Unable to get receiver {} frequency: {}", receiver_id, e);
                        None
                    }
                }
            },
            (Some(receiver_id), Command::setFrequency(freq)) => {
                let connection_string = &self.connection_strings()[receiver_id as usize];
                match TcpStream::connect(connection_string) {
                    Ok(mut stream) => {
                        let report = crate::rig::change_frequency(&mut stream, freq).unwrap();
                        Some(CommandResponseBody { response: report, receiver: Some(receiver_id) })
                    },
                    Err(e) => {
                        eprintln!("ERROR! Unable to set receiver {} frequency: {}", receiver_id, e);
                        None
                    }
                }
            },
            (Some(receiver_id), Command::getMode) => {
                let connection_string = &self.connection_strings()[receiver_id as usize];
                match TcpStream::connect(connection_string) {
                    Ok(mut stream) => {
                        let mode = crate::rig::get_mode(&mut stream).unwrap();
                        Some(CommandResponseBody { response: CommandResponse::ModeResponse(mode), receiver: Some(receiver_id) })
                    },
                    Err(e) => {
                        eprintln!("ERROR! Unable to get receiver {} mode: {}", receiver_id, e);
                        None
                    }
                }
            },
            (Some(receiver_id), Command::setMode(mode)) => {
                let connection_string = &self.connection_strings()[receiver_id as usize];
                match TcpStream::connect(connection_string) {
                    Ok(mut stream) => {
                        let report = crate::rig::change_mode(&mut stream, &mode).unwrap();
                        Some(CommandResponseBody { response: report, receiver: Some(receiver_id) })
                    },
                    Err(e) => {
                        eprintln!("ERROR! Unable to change receiver {} mode: {}", receiver_id, e);
                        None
                    }
                }
            },
            _ => {
                eprintln!("ERROR! Unexpected command: {}", msg);
                None
            }
        }
    }

    pub fn receivers(&self) -> Vec<ham_rs::Receiver> {
        let mut count = 0;
        self.config.receivers.iter().map(|(connection_string, _)| {
            let mut stream = TcpStream::connect(&connection_string).unwrap();
            let freq = crate::rig::get_frequency(&mut stream).unwrap();
            let mode = crate::rig::get_mode(&mut stream).unwrap();
            let ret = ham_rs::Receiver { id: count, frequency: freq.frequency, mode: mode.mode };
            count += 1;
            ret
        }).collect()
    }

    pub fn connection_strings(&self) -> Vec<String> {
        self.config.receivers.iter().map(|(connection_string, _)| {
            connection_string.to_string()
        }).collect()
    }
}