mod parser;
mod valkey;

//use std::io::{BufReader, BufRead};

//use parser::{RespParser, RespValue};


fn main() {
    let mut server = valkey::Valkey::new("127.0.0.1:6379".to_owned());
    server.run();
    //println!("Hello, world!");
    //let test = b"aklsj;df;lakj\r\n".to_vec();
    //
    //let cursor = std::io::Cursor::new(test);
    //
    //let mut buf = String::new();
    //let mut r = RespParser::new(cursor);
    //let out = r.reader.read_line(&mut buf);
    //println!("{out:?}");
    //
    //println!("{:?}", buf);
}
