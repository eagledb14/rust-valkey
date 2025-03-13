
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone)]
pub enum RespValue {
    SimpleString(String),
    Integer(i64),
    Error(String),
    BulkString(String),
    Array(Vec<RespValue>),
    NullBulkString,
    None
}

impl RespValue {
    pub fn unwrap(self) -> String {
        match self {
            RespValue::SimpleString(val) => val,
            RespValue::Integer(val) => val.to_string(),
            RespValue::Error(val) => val,
            RespValue::BulkString(val) => val,
            RespValue::NullBulkString => "".to_owned(),
            RespValue::None => "".to_owned(),
            RespValue::Array(val) => val.into_iter().map(|x| x.unwrap()).collect::<Vec<String>>().join(" ")

        }
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();

        match self {
            RespValue::Array(values) => {

                for val in values {
                    if let RespValue::Array(_) = val {
                        out.append(&mut val.flatten());
                    } else {
                        out.push(val.clone().unwrap());
                    }
                }
            },
            _ => {}
        }
        return out;
    }

   pub fn to_stream_output(&self) -> String {
        match self {
            RespValue::SimpleString(val) => format!("+{}\r\n", val),
            RespValue::Integer(val) => format!(":{}\r\n", val),
            RespValue::Error(val) => format!("-{}\r\n", val),
            RespValue::BulkString(val) => format!("${}\r\n{}\r\n", val.len(), val),
            RespValue::NullBulkString => "$-1\r\n".to_owned(),
            RespValue::None => "".to_owned(),
            RespValue::Array(val) => {
                let formatted_items = val.clone().into_iter()
                    .map(|x| x.to_stream_output()) // Recursively call `to_stream_output`
                    .collect::<Vec<String>>()
                    .join("");
                format!("*{}\r\n{}", val.len(), formatted_items)
            }
        }
    }}


//impl std::fmt::Debug for RespValue {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            RespValue::SimpleString(val) => format!("+{}", val)
//        }
//    }
//}

#[derive(Debug)]
pub struct RespParser<R> {
    pub reader: BufReader<R>,
}

impl <R: Read> RespParser<R> {
    pub fn new(reader: R) -> Self {
        return RespParser{
            reader: BufReader::new(reader),
        }
    }

    pub fn parse_input(&mut self) -> RespValue {
        let mut first_byte = [0u8; 1];
        match self.reader.read_exact(&mut first_byte) {
            Ok(_) => {},
            Err(_) => return RespValue::None
        }

        match first_byte[0] {
            b'+' => self.parse_simple_string(),
            b'-' => self.parse_errors(),
            b':' => self.parse_integers(),
            b'$' => self.parse_bulk_string(),
            b'*' => self.parse_array(),
            _ => {
                RespValue::None
            }
        }
    }

fn read_line(&mut self) -> String {
        let mut buf = String::new();
        let correct_parse = self.reader.read_line(&mut buf);
        match correct_parse {
            Ok(_) => {
                return buf.trim_end_matches("\r\n").to_string();

            }
            Err(_) => {
                return "".to_string();
            }
        }
    }

    fn parse_simple_string(&mut self) -> RespValue {
        let line = self.read_line();
        return RespValue::SimpleString(line);
    }

    fn parse_errors(&mut self) -> RespValue {
        let line = self.read_line();
        return RespValue::Error(line);
    }

    fn parse_integers(&mut self) -> RespValue {
        let line = self.read_line();
        match line.parse::<i64>() {
            Ok(i) => return RespValue::Integer(i),
            Err(_) => return RespValue::Integer(0)
        }
    }

    fn parse_bulk_string(&mut self) -> RespValue {
        // read the line
        let size_str = self.read_line();
        // get the input size
        if size_str == "-1" {
            return RespValue::NullBulkString;
        }

        let size = match size_str.parse::<u64>() {
            Ok(val) => val,
            Err(_) => 0
        };
        // read bytes until there are none left
        
        let mut buf = vec![0u8; size as usize];
        match self.reader.read_exact(&mut buf) {
            Ok(_) => {},
            Err(_) => {
                return RespValue::None
            }
        }

        return RespValue::BulkString(String::from_utf8(buf).unwrap());
    }

    fn parse_array(&mut self) -> RespValue {
        // read line
        // get array size
        let size = match self.read_line().parse::<usize>() {
            Ok(val) => val,
            Err(_) => 0
        };
        // do parse_input until the array size is done
        // return that array
        let mut items: Vec<RespValue> = Vec::with_capacity(size);

        for _ in 0..size {
            items.push(self.parse_input());
            self.read_line();
        }

        return RespValue::Array(items);
    }
}




#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use rand::prelude::*;

    use super::*;

    fn get_reader(input: String) -> RespParser<Cursor<Vec<u8>>> {
        let cursor = std::io::Cursor::new(input.into_bytes());

        RespParser::new(cursor)
    }

    fn get_integer(rng: &mut ThreadRng) -> (i64, String) {
        let integer = rng.random_range(i64::MIN..i64::MAX);
        return (integer, format!(":{}\r\n", integer));
    }

    fn get_simple_string(rng: &mut ThreadRng) -> (String, String) {
        let msg = (0..1000).map(|_| char::from(rng.random_range(32..126)));
        let msg_string: String = msg.collect();
        return (msg_string.clone(), format!("+{}\r\n",msg_string));
    }

    fn get_bulk_string(rng: &mut ThreadRng) -> (String, String) {
        let msg = (0..1000).map(|_| char::from(rng.random_range(0..126)));
        let msg_string: String = msg.collect();
        return (msg_string.clone(), format!("${}\r\n{}\r\n", msg_string.len(), msg_string));
    }

    #[test]
    fn reading() {
        let mut reader = get_reader("test input here\r\n".to_owned());

        assert_eq!("test input here", reader.read_line());
    }

    #[test]
    fn parse_integer() {
        let mut rng = rand::rng();
        
        for _ in  0..1_000 {
            let (integer, input) = get_integer(&mut rng);
            let mut reader = get_reader(input);

            let input = reader.parse_input();
            match input {
                RespValue::Integer(val) => assert_eq!(integer, val),
                _ => panic!("Incorrect parsed type {:?}", input)
            }

        }
    }

    #[test]
    fn parse_simple_string() {
        let mut rng = rand::rng();

        for _ in 0..1_000 {
            let (msg, input) = get_simple_string(&mut rng);
            let mut reader = get_reader(input);
            match reader.parse_input() {
                RespValue::SimpleString(val) => assert_eq!(msg, val),
                _ => panic!("Incorrect parsed type")
            }
        }
    }

    #[test]
    fn parse_bulk_string() {
        let mut rng = rand::rng();

        for _ in 0..1_000 {
            let (msg, input) = get_bulk_string(&mut rng);

            let mut reader = get_reader(input);
            match reader.parse_input() {
                RespValue::BulkString(val) => assert_eq!(msg, val),
                _ => panic!("Incorrect parsed type")
            }
        }

        let msg = format!("$-1\r\n\r\n");
        let mut reader = get_reader(msg);
        match reader.parse_input() {
            RespValue::NullBulkString => {},
            _ => panic!("Incorrect parsed type")
        }

    }

    #[test]
    fn parse_array() {
        let mut rng = rand::rng();
        let mut values = Vec::new(); 
        let mut items = Vec::new();

        for _ in 0..1_000 {
            let choice = rng.random_range(0..3);

            match choice {
                1 => {
                    let (integer, input) = get_integer(&mut rng);
                    values.push(integer.to_string());
                    items.push(input);
                }
                2 => {
                    let (msg, input) = get_simple_string(&mut rng);
                    values.push(msg);
                    items.push(input);
                }
                3 => {
                    let (msg, input) = get_bulk_string(&mut rng);
                    values.push(msg);
                    items.push(input);
                }
                _ => continue
            }

        }

        let input = format!("*{}\r\n{}", values.len(), items.join("\r\n"));
        let mut reader = get_reader(input);
        let output = reader.parse_input();

        if let RespValue::Array(array_values) = output {
            for (i, value) in array_values.iter().enumerate() {
                let str_value = match value {
                    RespValue::SimpleString(val) => val,
                    RespValue::BulkString(val) => &val,
                    RespValue::Integer(val) => &val.to_string(),
                    _ => {
                        panic!("bad type {:?}", value); 
                    },
                };

                assert_eq!(&values[i], str_value);
            }

        } else {
            panic!("Not an Array");
        }
    }

}
