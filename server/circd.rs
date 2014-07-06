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

extern crate circ_comms;
extern crate serialize;
extern crate time;

use serialize::json;
use std::io::{File, fs};
use std::io::net::unix::UnixListener;
use std::io::{Listener, Acceptor};
use std::os;

mod irc;
mod irc_channel;
mod irc_message;

///////////////////////////////////////////////////////////////////////////////
fn process_args() -> irc::ConnectionConfig
{
    match os::args().tail()
    {
        [ref arg] =>
        {
            let filename = Path::new(arg.as_slice());
            
            if !filename.exists()
            {
                fail!("File {} doesn't exist", arg);
            }
            
            let data = match File::open(&filename).read_to_end()
                {
                    Ok(d) => d.clone(),
                    Err(e) => fail!("Unable to read {}: {}", arg, e)
                };
            
            let string = std::str::from_utf8(data.as_slice()).unwrap();
            
            match json::decode::<irc::ConnectionConfig>(string.as_slice())
            {
                Ok(o)  => o,
                Err(e) => fail!("JSON decoding error: {}", e)
            }
        },
        _ => fail!("Configuration file must be specified")
    }
}

///////////////////////////////////////////////////////////////////////////////
fn main()
{
    let config = process_args();

    let connection = irc::Connection::new(config);

    let socket = Path::new(circ_comms::address());
    if socket.exists()
    {
        match fs::unlink(&socket)
        {
            Ok(_)  => (),
            Err(e) => fail!("Unable to remove {}: {}", circ_comms::address(), e)
        }
    }

    let stream = UnixListener::bind(&socket);
    
    for c in stream.listen().incoming()
    {
        let mut client = match c
            {
                Ok(x) => x,
                Err(e) => { println!("Failed to get client: {}", e); continue }
            };

        let request = match circ_comms::read_request(&mut client)
            {
                Some(r) => r,
                None => continue
            };
        
        match request
        {
            circ_comms::ListChannels => 
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::GetStatus =>
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::GetMessages(_) =>
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::GetUsers(_) => (),
            circ_comms::Join(_) => connection.request(request),
            circ_comms::Part(_) => connection.request(request),
            circ_comms::SendMessage(_, _) => connection.request(request),
            circ_comms::Quit => {connection.request(request); break} // not a clean quit, but it works
        }
    }
}
