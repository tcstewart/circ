#![crate_id="circ_comms"]
#![crate_type = "rlib"]

// Copyright 2014 tcstewart
// This file is part of circ.
// circ is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// circ is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with circ.  If not, see <http://www.gnu.org/licenses/>.

///////////////////////////////////////////////////////////////////////////////
extern crate serialize;
extern crate time;

///////////////////////////////////////////////////////////////////////////////
use serialize::{json, Encodable, Decodable};
use std::io::net::unix::UnixStream;
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
    GetStatus(String),
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
    pub fn new(time: Timespec, user: String, msg: String) -> Message
    {
        Message{time: time, user: user, msg: msg}
    }
}

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show, Decodable, Encodable)]
pub enum Response
{
    Channels(Vec<String>),
    Status(uint),
    Messages(Vec<Message>),
    Users(Vec<String>),
    Error(String)
}

///////////////////////////////////////////////////////////////////////////////
fn decode_request(data: &str) -> Request
{
    let json_object = json::from_str(data.as_slice());
    let mut decoder = json::Decoder::new(json_object.unwrap());
    let request: Request = match Decodable::decode(&mut decoder)
        {
            Ok(o)  => o,
            Err(e) => fail!("JSON decoding error: {}", e)
        };
    
    request
}

///////////////////////////////////////////////////////////////////////////////
fn decode_response(data: &str) -> Response
{
    let json_object = json::from_str(data.as_slice());
    let mut decoder = json::Decoder::new(json_object.unwrap());
    let response: Response = match Decodable::decode(&mut decoder)
        {
            Ok(o)  => o,
            Err(e) => fail!("JSON decoding error: {}", e)
        };
    
    response
}

///////////////////////////////////////////////////////////////////////////////
pub fn read_request(stream: &mut UnixStream) -> Request
{
    let len = stream.read_be_uint().unwrap();
    let data = stream.read_exact(len).unwrap();

    let string = match ::std::str::from_utf8(data.as_slice())
        {
            Some(s) => s,
            None => fail!("Unable to convert data to string")
        };

    decode_request(string)
}

///////////////////////////////////////////////////////////////////////////////
pub fn write_request(stream: &mut UnixStream, request: &Request)
{
    let string =  json::Encoder::str_encode(&request);
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
    let string =  json::Encoder::str_encode(&response);
    let data = string.as_bytes();
    
    stream.write_be_uint(data.len()).unwrap();
    stream.write(data).unwrap();
}


