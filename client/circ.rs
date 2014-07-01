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
extern crate term;
extern crate time;

///////////////////////////////////////////////////////////////////////////////
use circ_comms::Message;
use std::io::net::unix::UnixStream;
use std::os;


///////////////////////////////////////////////////////////////////////////////
fn process_args() -> (circ_comms::Request, bool)
{
   let opts = 
        [
            getopts::optflag("l", "list-channels", "List the channels that circd is using"),
            getopts::optopt("c", "channel", "Channel to use for the operations", "#rust"),
            getopts::optflag("j", "join", "Join a channel"),
            getopts::optflag("m", "msg", "Send a message to a channel"),
            getopts::optflag("p", "part", "Part from a channel"),
            getopts::optflag("q", "quit", "Quit irc and stop circd"),
            getopts::optflag("s", "status", "Get the unread message status of all channels"),
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


    let data = if matches.free.is_empty()
               {
                   None
               }
               else
               {
                   Some(matches.free.connect(" "))
               };
       
    let (request, response_expected) = match *flags.get(0)
        {
            "l" => (circ_comms::ListChannels, true),
            "j" => (circ_comms::Join(channel.unwrap()), false),
            "m" => (circ_comms::SendMessage(channel.unwrap(), data.unwrap()), false),
            "p" => (circ_comms::Part(channel.unwrap()), false),
            "q" => (circ_comms::Quit, false),
            "s" => (circ_comms::GetStatus, true),
            "u" => (circ_comms::GetMessages(channel.unwrap()), true),
            x   => fail!("Unknown option {}",x )
        };

    (request, response_expected)
}

///////////////////////////////////////////////////////////////////////////////
fn print_msgs(msgs: &Vec<Message>)
{
    let mut t = term::stdout().unwrap();

    for m in msgs.iter()
    {
        (write!(t, "[")).unwrap();
        t.fg(term::color::MAGENTA).unwrap();
        (write!(t, "{}", time::at(m.time).strftime("%T"))).unwrap();
        t.reset().unwrap();
        (write!(t, "] ")).unwrap();

        t.fg(term::color::GREEN).unwrap();
        (write!(t, "{}", m.user.as_slice().split('!').next().unwrap())).unwrap();

        t.reset().unwrap();
        (writeln!(t, " {}", m.msg)).unwrap();

    }
}

///////////////////////////////////////////////////////////////////////////////
fn main()
{
    let (request, response_expected) = process_args();
    
    let socket = Path::new(circ_comms::address());

    if socket.exists().not()
    {
        fail!("Socket {} doesn't exist", circ_comms::address());
    }

    let mut stream = UnixStream::connect(&socket).unwrap();

    circ_comms::write_request(&mut stream, &request);

    if response_expected
    {
        let response = circ_comms::read_response(&mut stream);

        match response
        {
            circ_comms::Channels(channels) => println!("{}", channels),
            circ_comms::Messages(m) => print_msgs(&m),
            circ_comms::Status(s) => 
            {
                for t in s.iter()
                {
                    let (channel, count) = t.clone();
                    if count == 1
                    {
                        println!("{} has 1 new message", channel);
                    }
                    else if count > 1
                    {
                        println!("{} has {} new messages", channel, count);
                    }
                }
            },
            r => fail!("Unexpected response{}", r)
        }
    }
}
