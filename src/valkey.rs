use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, Write};
use std::thread;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use crate::parser::{RespParser, RespValue};

pub struct Valkey {
    listener: TcpListener,
    host: String,
    db: Arc<RwLock<HashMap<String, RespValue>>>,
}

impl Valkey {
    pub fn new(host: String) -> Self {
        return Valkey {
            listener: TcpListener::bind(host.clone()).unwrap(),
            host,
            db: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn run(&mut self) {
        println!("listening on: {}", self.host);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db_clone = self.db.clone();
                    thread::spawn(|| {
                        Self::handle_client(db_clone, stream);
                    });
                },
                Err(_) => {}
            }
        }
    }

    fn handle_client(db: Arc<RwLock<HashMap<String, RespValue>>>, mut stream: TcpStream) {
        let mut writer = stream.try_clone().unwrap();
        let reader = BufReader::new(&mut stream);

        let mut parser = RespParser::new(reader);
        loop {
            let input = parser.parse_input();


            match input {
                RespValue::None => return,
                _ => {
                    Self::handle_command(db.clone(), &mut writer, input);
                }
            }

        }
    }

    fn handle_command(db: Arc<RwLock<HashMap<String, RespValue>>>, writer: &mut TcpStream, command: RespValue) {
        let response: String;
        println!("{:?}", command.clone().flatten());

        if let RespValue::Array(arr) = command.clone() {
            let cmd = command.flatten();
            match cmd[0].as_str() {
                "ECHO" => response = format!("+{}\r\n",cmd[1]),
                "PING" => response = "+PONG\r\n".to_owned(),
                "set" => {
                    let mut map = db.write().unwrap();
                    map.insert(arr[1].clone().unwrap(), arr[2].clone());

                    response = "+CHEESE\r\n".to_owned()
                }
                "get" => {
                    let map = db.read().unwrap();

                    match map.get(&arr[1].clone().unwrap()) {
                        Some(val) => response = val.clone().to_stream_output(),
                        None => response = RespValue::NullBulkString.to_stream_output(),
                    }

                }
                _ => response = "+OK\r\n".to_owned()
            }
        } else {
            response = "+OK\r\n".to_owned()
        }

        let _output = writer.write_all(&response.into_bytes());
    }
}


#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use std::net::TcpStream;
    use std::io::Write;


    use super::*;


    fn get_tcp_client(host: String) -> TcpStream {
        let stream = TcpStream::connect(host);

        match stream {
            Ok(s) => return s,
            Err(_) => panic!("failed to open stream"),
        }
    }

    #[test]
    fn set_value() {
        let host = "127.0.0.1:8000".to_owned();
        let valkey = Valkey::new(host.clone());
        let mut client = get_tcp_client(host);


        let written = client.write_all(b"*3\r\n\n+set\r\n\n+name\r\n\n+jim\r\n\n");
        println!("written: {:?}", written);


        thread::sleep(Duration::from_millis(100));
        let map = valkey.db.read().unwrap();
        println!("{:?}", map.get("name"));
        //let name = valkey.db.lock().unwrap().get("name").unwrap().clone().unwrap();
        //assert_eq!("jim", name);
    }

}
