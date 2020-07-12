use ws::{Handler, Message, Request, Response, Sender};
use std::str;
use crate::proxy::ProxyService;

pub struct Server {
    pub out: Sender,
    pub config: crate::Config,
}

impl Handler for Server {
    //
    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        let pieces : Vec<&str> = req.resource().split("/").collect();
        
        match pieces.as_slice() {
            ["","ws"] => Response::from_request(req),
            _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            ws::Message::Text(msg) => {
                println!("<< {}", msg);
                match ProxyService::handle_command(&self.config, &msg) {
                    Some(resp) => {
                        let resp = serde_json::to_string(&resp).unwrap();
                        println!(">> {}", resp);
                        self.out.send(resp).unwrap();
                    },
                    None => {
                        eprintln!("No Response!");
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