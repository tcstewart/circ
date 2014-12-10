#![feature(phase)]

///////////////////////////////////////////////////////////////////////////////
extern crate collections;
extern crate getopts;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate term;
extern crate time;

extern crate circ_comms;

///////////////////////////////////////////////////////////////////////////////
use circ_comms::Message;
use collections::bitv::Bitv;
use std::io::net::pipe::UnixStream;
use std::os;
use std::io::fs::PathExtensions;

///////////////////////////////////////////////////////////////////////////////
fn process_args() -> (circ_comms::Request, bool, Vec<String>)
{
   let opts : &[getopts::OptGroup] = 
        &[
            getopts::optflag("l", "list-channels", "List the channels that circd is using"),
            getopts::optopt("c", "channel", "Channel to use for the operations", "#rust"),
            getopts::optflag("j", "join", "Join a channel"),
            getopts::optflag("m", "msg", "Send a message to a channel"),
            getopts::optflag("p", "part", "Part from a channel"),
            getopts::optflag("q", "quit", "Quit irc and stop circd"),
            getopts::optflag("s", "status", "Get the unread message status of all channels"),
            getopts::optflag("u", "unread", "Get the unread messages from a channel"),
            getopts::optflag("w", "who", "Get the users currently active on the channel"),
            getopts::optopt("h", "highlight", "List of words that would cause the line to be highlighted", "word1[,word2...]")
        ];
    
    let matches = match getopts::getopts(os::args().tail(), opts)
        {
            Ok(m) => m,
            Err(e) => panic!("Invalid options\n{}", e)
        };

    let channel = matches.opt_str("channel");

    let v = ["l", "j", "m", "p", "q", "s", "u", "w"];
    
    let flags : Vec<&str> = v.iter().filter(|&x| matches.opt_present(*x))
                             .map(|x| x.as_slice()).collect();

    if flags.len() > 1 || flags.len() == 0
    {
        panic!("Must specify one of [l, j, m, p, q, s, u, w]");
    }

    let highlights : Vec<String> = match matches.opt_str("highlight")
        {
            Some(s) => s.as_slice().split(',').map(|x| x.to_string()).collect(),
            None => Vec::new()
        };


    let data = if matches.free.is_empty()
               {
                   None
               }
               else
               {
                   Some(matches.free.connect(" "))
               };
       
    match flags[0]
    {
        "l" => (circ_comms::Request::ListChannels, true, highlights),
        "j" => (circ_comms::Request::Join(channel.unwrap()), false, highlights),
        "m" => (circ_comms::Request::SendMessage(channel.unwrap(), data.unwrap()), false, highlights),
        "p" => (circ_comms::Request::Part(channel.unwrap()), false, highlights),
        "q" => (circ_comms::Request::Quit, false, highlights),
        "s" => (circ_comms::Request::GetStatus, true, highlights),
        "u" => (circ_comms::Request::GetMessages(channel.unwrap()), true, highlights),
        x   => panic!("Unknown option {}", x)
    }
}

///////////////////////////////////////////////////////////////////////////////
fn print_msgs(msgs: &Vec<Message>, highlights: &Vec<String>)
{
    let mut t = term::stdout().unwrap();
    
    let re = regex!(r"\001ACTION (?P<action>[^\001]+)\001");
    
    for m in msgs.iter()
    {
        // TODO: There has to be a better way of doing this...
        let vec : Vec<bool> = highlights.iter().map(|x| m.msg.as_slice().contains(x.as_slice())).collect();
        let bvec: Bitv = vec.iter().map(|n| *n).collect();
        
        let highlight = bvec.any();

        (write!(t, "[")).unwrap();
        t.fg(term::color::MAGENTA).unwrap();
        (write!(t, "{}", time::at(m.time).strftime("%T").unwrap())).unwrap();
        t.reset().unwrap();
        (write!(t, "] ")).unwrap();

        let user = m.user.as_slice().split('!').next().unwrap();
        

        match re.captures(m.msg.as_slice())
        {
            Some(c) => 
                { 
                    t.fg(term::color::BLUE).unwrap();
                    (writeln!(t, "{} {}", user, c.name("action"))).unwrap();
                    t.reset().unwrap();
                },
            None =>
                {
                    t.fg(term::color::GREEN).unwrap();
                    (write!(t, "{}", user)).unwrap();
                    t.reset().unwrap();
                    (write!(t, "> ")).unwrap();
                    if highlight
                    {
                        t.bg(term::color::BLUE).unwrap();
                    }
                    (write!(t, "{}", m.msg)).unwrap();
                    t.reset().unwrap();
                    (writeln!(t, "")).unwrap();
                    
                }
        };

    }
}

///////////////////////////////////////////////////////////////////////////////
fn main()
{
    let (request, response_expected, highlights) = process_args();
    
    let socket = Path::new(circ_comms::address());

    if socket.exists().not()
    {
        panic!("Socket {} doesn't exist", circ_comms::address());
    }

    let mut stream = UnixStream::connect(&socket).unwrap();

    circ_comms::write_request(&mut stream, &request);

    if response_expected
    {
        let response = circ_comms::read_response(&mut stream);

        match response
        {
            circ_comms::Response::Channels(channels) => println!("{}", channels),
            circ_comms::Response::Messages(m) => print_msgs(&m, &highlights),
            circ_comms::Response::Status(s) => 
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
            r => panic!("Unexpected response{}", r)
        }
    }
}
