#![feature(slicing_syntax)]
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
extern crate irc;
extern crate time;

use irc::data::config::Config;

use std::io::fs;
use std::io::net::pipe::UnixListener;
use std::io::{Listener, Acceptor};
use std::os;
use std::io::fs::PathExtensions;

mod connection;
mod irc_channel;

///////////////////////////////////////////////////////////////////////////////
fn process_args() -> Config
{
    match os::args().tail()
    {
        [ref arg] =>
        {
            let filename = Path::new(arg.as_slice());
            
            if !filename.exists()
            {
                panic!("File {} doesn't exist", arg);
            }

            Config::load(filename).unwrap()
        },
        _ => panic!("Configuration file must be specified")
    }
}

///////////////////////////////////////////////////////////////////////////////
fn main()
{
    let config = process_args();

    let connection = connection::Connection::new(config);
    
    let socket = Path::new(circ_comms::address());
    if socket.exists()
    {
        match fs::unlink(&socket)
        {
            Ok(_)  => (),
            Err(e) => panic!("Unable to remove {}: {}", circ_comms::address(), e)
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
            circ_comms::Request::ListChannels => 
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::Request::GetStatus =>
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::Request::GetMessages(_) =>
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::Request::GetUsers(_) => (),
            circ_comms::Request::Join(_) => connection.request(request),
            circ_comms::Request::Part(_) => connection.request(request),
            circ_comms::Request::SendMessage(_, _) => connection.request(request),
            circ_comms::Request::Quit => {connection.request(request); break} // not a clean quit, but it works
        }
    }
}
