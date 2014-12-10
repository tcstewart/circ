#![crate_name="circ_comms"]
#![crate_type = "dylib"]

///////////////////////////////////////////////////////////////////////////////
extern crate serialize;
extern crate time;

///////////////////////////////////////////////////////////////////////////////
use serialize::json;
use std::io::net::pipe::UnixStream;
use std::os;
use time::Timespec;

///////////////////////////////////////////////////////////////////////////////
pub fn address() -> String
{
    match os::getenv("HOME")
    {
        Some(val) => format!("{}/.circd/circd-socket", val),
        None      => format!("/tmp/circd-socket")
    }
}

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show, Decodable, Encodable)]
pub enum Request
{
    ListChannels,
    GetStatus,
    GetMessages(String),
    GetUsers(String),
    Join(String),
    Part(String),
    SendMessage(String, String),
    Quit
}

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show, Decodable, Encodable)]
pub struct Message
{
    pub time: Timespec,
    pub user: String,
    pub msg:  String
}

///////////////////////////////////////////////////////////////////////////////
impl Message
{
    pub fn new(time: Timespec, user: &str, msg: &str) -> Message
    {
        Message{time: time, user: user.to_string(), msg: msg.to_string()}
    }
}

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show, Decodable, Encodable)]
pub enum Response
{
    Channels(Vec<String>),
    Status(Vec<(String, uint)>),
    Messages(Vec<Message>),
    Users(Vec<String>),
    Error(String)
}

///////////////////////////////////////////////////////////////////////////////
fn decode_request(data: &str) -> Option<Request>
{
    match json::decode::<Request>(data.as_slice())
    {
        Ok(o)  => Some(o),
        Err(e) => { println!("JSON decoding error: {}", e); None }
    }
}

///////////////////////////////////////////////////////////////////////////////
fn decode_response(data: &str) -> Response
{
    let response: Response = match json::decode(data.as_slice())
        {
            Ok(o)  => o,
            Err(e) => panic!("JSON decoding error: {}", e)
        };
    
    response
}

///////////////////////////////////////////////////////////////////////////////
pub fn read_request(stream: &mut UnixStream) -> Option<Request>
{
    let len = stream.read_be_uint().unwrap();
    let data = stream.read_exact(len).unwrap();

    match ::std::str::from_utf8(data.as_slice())
    {
        Some(s) => decode_request(s),
        None => { println!("Failed to read string from data"); None }
    }
}

///////////////////////////////////////////////////////////////////////////////
pub fn write_request(stream: &mut UnixStream, request: &Request)
{
    let string =  json::encode(&request);
    let data = string.as_bytes();
    
    stream.write_be_uint(data.len()).unwrap();
    stream.write(data).unwrap();
}

///////////////////////////////////////////////////////////////////////////////
pub fn read_response(stream: &mut UnixStream) -> Response
{
    let len = stream.read_be_uint().unwrap();
    let data = stream.read_exact(len).unwrap();

    let string = ::std::str::from_utf8(data.as_slice()).unwrap();

    decode_response(string)
}

///////////////////////////////////////////////////////////////////////////////
pub fn write_response(stream: &mut UnixStream, response: Response)
{
    let string =  json::encode(&response);
    let data = string.as_bytes();
    
    stream.write_be_uint(data.len()).unwrap();
    stream.write(data).unwrap();
}


