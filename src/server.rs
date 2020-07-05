use ws::{Handler, Message, Request, Response, Sender};
use std::net::{TcpStream};
use std::str;
use ham_rs::{Command,CommandResponse,CommandResponseBody,CommandMessage};

pub struct Server {
    pub out: Sender,
    pub stream: TcpStream,
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
                println!("Received Command: {}", msg);
                let cmd : CommandMessage = serde_json::from_str(&msg).unwrap();
                let mut stream = self.stream.try_clone().unwrap();
                match (cmd.receiver, cmd.cmd) {
                    (None, Command::getReceivers) => {
                        let freq = crate::rig::get_frequency(&mut stream).unwrap();
                        let mode = crate::rig::get_mode(&mut stream).unwrap();
                        let receiver = ham_rs::Receiver { id: 0, frequency: freq.frequency, mode: mode.mode };
                        let resp = CommandResponseBody { response: CommandResponse::ReceiverList(vec![receiver]), receiver: None };
                        let j = serde_json::to_string(&resp).unwrap();
                        self.out.send(j).unwrap();
                    },
                    (Some(receiver_id), Command::getFrequency) => {
                        let freq = crate::rig::get_frequency(&mut stream).unwrap();
                        let resp = CommandResponseBody { response: CommandResponse::FrequencyResponse(freq), receiver: Some(receiver_id) };
                        let j = serde_json::to_string(&resp).unwrap();
                        self.out.send(j).unwrap();
                    },
                    (Some(receiver_id), Command::setFrequency(freq)) => {
                        let report = crate::rig::change_frequency(&mut stream, freq).unwrap();
                        let resp = CommandResponseBody { response: report, receiver: Some(receiver_id) };
                        let j = serde_json::to_string(&resp).unwrap();
                        self.out.send(j).unwrap();
                    },
                    (Some(receiver_id), Command::getMode) => {
                        let mode = crate::rig::get_mode(&mut stream).unwrap();
                        let resp = CommandResponseBody { response: CommandResponse::ModeResponse(mode), receiver: Some(receiver_id) };
                        let j = serde_json::to_string(&resp).unwrap();
                        self.out.send(j).unwrap();
                    },
                    (Some(receiver_id), Command::setMode(mode)) => {
                        let report = crate::rig::change_mode(&mut stream, &mode).unwrap();
                        let resp = CommandResponseBody { response: report, receiver: Some(receiver_id) };
                        let j = serde_json::to_string(&resp).unwrap();
                        self.out.send(j).unwrap();
                    },
                    _ => {
                        eprintln!("ERROR! Unexpected command: {}", msg);
                    }
                }
            },
            _ => {
                eprintln!("ERROR! unexpected request: {:?}", msg);
            }
        }
        Ok(())
    }
}