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
extern crate time;

///////////////////////////////////////////////////////////////////////////////
use circ_comms::Message;
use std::io::net::unix::UnixStream;
use std::os;


///////////////////////////////////////////////////////////////////////////////
fn process_args() -> circ_comms::Request
{
   let opts = 
        [
            getopts::optflag("l", "list-channels", "List the channels that circd is using"),
            getopts::optopt("c", "channel", "Channel to use for the operations", "#rust"),
            getopts::optflag("j", "join", "Join a channel"),
            getopts::optflag("m", "msg", "Send a message to a channel"),
            getopts::optflag("p", "part", "Part from a channel"),
            getopts::optflag("q", "quit", "Quit irc and stop circd"),
            getopts::optflag("s", "status", "Get the status of a channel"),
            getopts::optflag("u", "unread", "Get the unread messages from a channel"),
            getopts::optflag("w", "who", "Get the users currently active on the channel")
        ];
    
    let matches = match getopts::getopts(os::args().tail(), opts)
        {
            Ok(m) => m,
            Err(e) => fail!("Invalid options\n{}", e)
        };

    let channel = matches.opt_str("channel");

    let v = ["l", "j", "m", "p", "q", "s", "u", "w"];
    
    let flags : Vec<&str> = v.iter().filter(|&x| matches.opt_present(*x))
                             .map(|&x| std::str::from_utf8(x.as_bytes()).unwrap()).collect();

    if flags.len() > 1 || flags.len() == 0
    {
        fail!("Must specify one of [l, j, m, p, q, s, u, w]");
    }

    let command = match *flags.get(0)
        {
            "l" => circ_comms::GetChannelList,
            "j" => circ_comms::JoinChannel,
            "m" => circ_comms::SendMessage,
            "p" => circ_comms::PartChannel,
            "q" => circ_comms::QuitIrc,
            "s" => circ_comms::GetChannelStatus,
            "u" => circ_comms::GetChannelMessages,
            x   => fail!("Unknown option {}",x )
        };

    let data = if matches.free.is_empty()
               {
                   None
               }
               else
               {
                   Some(matches.free.connect(" "))
               };
       

    circ_comms::Request{command: command, channel: channel, data: data}
}

fn print_msgs(msgs: &Vec<Message>)
{
    for m in msgs.iter()
    {
        println!("[{}] {}> {}",
                 time::at(m.time).strftime("%T"),
                 m.user.as_slice().split('!').next().unwrap(),
                 m.msg);
    }
}

fn main()
{
    let request = process_args();
    
    let socket = Path::new(circ_comms::address());

    if socket.exists().not()
    {
        fail!("Socket {} doesn't exist", circ_comms::address());
    }

    let mut stream = UnixStream::connect(&socket).unwrap();

    circ_comms::write_request(&mut stream, &request);

    match request.command
    {
        circ_comms::GetChannelList =>
            {
                let response = circ_comms::read_response(&mut stream);
                match response
                {
                    circ_comms::Channels(channels) =>
                        println!("{}", channels),
                    r => fail!("Unexpected response {}", r)
                }
            },
        circ_comms::GetChannelMessages =>
            {
                let response = circ_comms::read_response(&mut stream);
                match response
                {
                    circ_comms::Messages(m) => print_msgs(&m),
                    r => fail!("Unexpected response{}", r)
                }
            },
        circ_comms::GetChannelStatus =>
            {
                let response = circ_comms::read_response(&mut stream);
                match response
                {
                    circ_comms::Status(s) => println!("{} has {} new messages",
                                                      request.channel.unwrap(), s),
                    r => fail!("Unexpected response{}", r)
                }
            },
        _ => ()
    }
}
