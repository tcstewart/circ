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
extern crate getopts;
extern crate serialize;
extern crate time;

use std::io::fs;
use std::io::net::unix::UnixListener;
use std::io::{Listener, Acceptor};
use std::os;

mod irc;
mod irc_channel;
mod irc_message;

///////////////////////////////////////////////////////////////////////////////
fn process_args() -> irc::ConnectionConfig
{
   let opts = 
        [
            getopts::reqopt("n", "nick", "Nick name", "bob"),
            getopts::reqopt("r", "realname", "Real name", "Bob Smith"),
            getopts::optopt("p", "port", "Port number", "6667")
        ];
    
    let matches = match getopts::getopts(os::args().tail(), opts)
        {
            Ok(m) => m,
            Err(f) => fail!("Invalid options\n{}", f)
        };

    let nick = match matches.opt_str("nick")
        {
            Some(nick) => nick,
            None => fail!("Invalid nick provided")
        };

    let realname = match matches.opt_str("realname")
        {
            Some(name) => name,
            None => fail!("Invalid name provided")
        };

    let port = match matches.opt_str("port")
        {
            Some(s) => match from_str::<u16>(s.as_slice())
                {
                    Some(n) => n,
                    None => fail!("Invalid port number")
                },
            None => 6667
        };

    if matches.free.is_empty()
    {
        fail!("No address specified");
    }

    let address = from_str(matches.free.get(0).as_slice()).unwrap();

    irc::ConnectionConfig::new(address, port, nick, realname)
}

///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
fn main()
{
    let config = process_args();

    let connection = irc::Connection::new(config);

    let socket = Path::new(circ_comms::address());
    if socket.exists()
    {
        fs::unlink(&socket).unwrap();
    }
    let stream = UnixListener::bind(&socket);
    
    // TODO: need much better input validation...
    for c in stream.listen().incoming()
    {
        let mut client = c.unwrap();
        let request = circ_comms::read_request(&mut client);
        
        match request
        {
            circ_comms::ListChannels => 
                circ_comms::write_response(&mut client,
                                           connection.request_response(request)),
            circ_comms::GetStatus(_) =>
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
