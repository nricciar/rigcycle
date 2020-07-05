use ws::{Handler, Message, Request, Response, Sender};
use std::net::{TcpStream};
use std::str;
use ham_rs::{Command,CommandResponse,CommandResponseBody,CommandMessage};

pub struct Server {
    pub out: Sender,
    pub config: crate::Config,
}
impl Server {
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

impl Handler for Server {
    //
    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        let pieces : Vec<&str> = req.resource().split("/").collect();
        
        //match req.resource() {
        match pieces.as_slice() {
            ["","ws"] => Response::from_request(req),
            _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            ws::Message::Text(msg) => {
                println!("<< {}", msg);
                let cmd : CommandMessage = serde_json::from_str(&msg).unwrap();
                //let mut stream = self.stream.try_clone().unwrap();
                let resp =
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
                    };
                match resp {
                    Some(resp) => {
                        let resp = serde_json::to_string(&resp).unwrap();
                        println!(">> {}", resp);
                        self.out.send(resp).unwrap();
                    },
                    None => {}
                }
            },
            _ => {
                eprintln!("ERROR! unexpected request: {:?}", msg);
            }
        }
        Ok(())
    }
}