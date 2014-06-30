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

use std::collections::HashMap;
use std::io::BufferedStream;
use std::io::net::tcp::TcpStream;
use std::string::String;

use circ_comms;
use circ_comms::{Request, Response};
use irc_channel;
use irc_message::Message;

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show, Decodable)]
pub struct ConnectionConfig
{
    address: String,
    port: u16,
    nickname: String,
    realname: String
}

///////////////////////////////////////////////////////////////////////////////
pub struct Connection
{
    process_tx: Sender<Request>,
    process_rx: Receiver<Response>
}

///////////////////////////////////////////////////////////////////////////////
fn chomp(buffer: &mut Vec<u8>)
{
    // TODO: remove unwrap()
    let lf = buffer.pop();
    let cr = buffer.pop();

    if lf.unwrap() != '\n' as u8 && cr.unwrap() != '\r' as u8
    {
        println!("{}{}", cr, lf);
        fail!("Failed to find \\r\\n");
    }
}

///////////////////////////////////////////////////////////////////////////////
fn rx_task(stream: TcpStream,
           tx: Sender<Message>)
{
    spawn(proc()
          {
              let mut rx = BufferedStream::new(stream); 
              loop
              {
                  let mut buffer = match rx.read_until('\n' as u8)
                  {
                      Ok(s) => s,
                      Err(_) => break
                  };
                  
                  chomp(&mut buffer);
                  
                  let msg = Message::parse(buffer.as_slice());
                  
                  tx.send(msg);
              }
          });
}

///////////////////////////////////////////////////////////////////////////////
fn tx_task(rx: Receiver<String>,
           stream: TcpStream)
{
    spawn(proc()
          {
              let mut tx = stream.clone();
              for msg in rx.iter()
              {
                  tx.write_str(msg.as_slice()).unwrap();
              }
          });
}

///////////////////////////////////////////////////////////////////////////////
fn set_topic(channels: &mut HashMap<String, irc_channel::Channel>, msg: Message)
{
    let name = msg.parameters.get(0).clone();

    let topic = match msg.trailing
        {
            Some(t) => t,
            None    => "No topic provided".to_string()
        };

    channels.insert_or_update_with(name.clone(),
                                   {
                                       let mut c = irc_channel::Channel::new(name);
                                       c.set_topic(topic.clone());
                                       c
                                   },
                                   |_, c| c.set_topic(topic.clone()));
}

///////////////////////////////////////////////////////////////////////////////
fn add_message(channels: &mut HashMap<String, irc_channel::Channel>, msg: Message)
{
    let name = msg.parameters.get(0).clone();

    if name == "AUTH".to_string() { return (); }

    channels.insert_or_update_with(name.clone(),
                                   {
                                       let mut c = irc_channel::Channel::new(name);
                                       c.add(msg.clone());
                                       c
                                   },
                                   |_, c| c.add(msg.clone()));
}

///////////////////////////////////////////////////////////////////////////////
fn get_channels(channels: &HashMap<String, irc_channel::Channel>) -> Response
{
    let mut names : Vec<String> = Vec::new();

    for name in channels.keys()
    {
        names.push(name.clone());
    }

    circ_comms::Channels(names)
}

///////////////////////////////////////////////////////////////////////////////
fn get_messages(channels: &mut HashMap<String, irc_channel::Channel>, name: &String) -> Response
{
    let channel = channels.find_mut(name);

    match channel
    {
        Some(c) =>
        {
            let mut r = Vec::new();
            for m in c.messages.iter()
            {
                // TODO: I need to understand the borrowing much better...
                let user = match m.prefix.clone()
                    {
                        Some(p) => p,
                        None    => "Unknown User".to_string()
                    };
                let msg = match m.trailing.clone()
                    {
                        Some(m) => m,
                        None    => "No message".to_string()
                    };

                r.push(circ_comms::Message::new(m.time,
                                                user,
                                                msg));
            }
            c.clear();
            circ_comms::Messages(r)
        },
        None    => circ_comms::Error(format!("Unknown channel {}", name))
    }
    
}

///////////////////////////////////////////////////////////////////////////////
fn get_status(channels: &HashMap<String, irc_channel::Channel>) -> Response
{
    let mut statuses: Vec<(String, uint)> = Vec::new();

    for (name, channel) in channels.iter()
    {
        statuses.push((name.to_string(), channel.messages.len()));
    }
    
    circ_comms::Status(statuses)
    
}
///////////////////////////////////////////////////////////////////////////////
fn process_task(rx: Receiver<Message>,
                tx: Sender<String>,
                response_tx: Sender<Response>, 
                request_rx: Receiver<Request>,
                nickname: String,
                realname: String)
{
    spawn(proc()
          {
              let mut channels = HashMap::new();

              // send user information to server
              tx.send(Message::nick(&nickname));
              tx.send(Message::user(&realname));

              loop
              {
                  select!(msg = rx.recv() =>
                          match msg.command.as_slice()
                          {
                              "ERROR"   => {println!("Error... {}", msg);},
                              "PING"    => tx.send(Message::pong(&msg.trailing.unwrap())),
                              "TOPIC"   => set_topic(&mut channels, msg),
                              "PRIVMSG"|"NOTICE" => add_message(&mut channels, msg),
                              _         => () //println!("{}", msg)
                          },
                          request = request_rx.recv() =>
                          match request
                          {
                              circ_comms::ListChannels => response_tx.send(get_channels(&channels)),
                              circ_comms::GetStatus => response_tx.send(get_status(&channels)),
                              circ_comms::GetMessages(channel) =>
                                  response_tx.send(get_messages(&mut channels, &channel)),
                              circ_comms::GetUsers(_) => response_tx.send(circ_comms::Users(Vec::new())),
                              circ_comms::Join(channel) => tx.send(Message::join(&channel)),
                              circ_comms::Part(channel) => tx.send(Message::part(&channel)),
                              circ_comms::SendMessage(channel, msg) => tx.send(Message::msg(&channel, &msg)),
                              circ_comms::Quit => {tx.send(Message::quit()); break}
                          });
              }
          });
}

///////////////////////////////////////////////////////////////////////////////
impl Connection
{
    ///////////////////////////////////////////////////////////////////////////
    pub fn new(config: ConnectionConfig) -> Connection
    {
        
        let stream = TcpStream::connect(config.address.as_slice(), config.port).unwrap();

        // channels to handle communication with tasks servicing the irc server
        let (incoming_msg_tx, incoming_msg_rx) = channel();
        let (outgoing_msg_tx, outgoing_msg_rx) = channel();

        // duplex to handle communication with process task
        let ((request_tx, request_rx), (response_tx, response_rx)) = (channel(), channel());

        // Start up the tasks to communicate with the irc server
        rx_task(stream.clone(), incoming_msg_tx);
        tx_task(outgoing_msg_rx, stream);

        process_task(incoming_msg_rx, outgoing_msg_tx, response_tx, request_rx,
                     config.nickname.clone(), config.realname.clone());
        
        Connection{process_tx: request_tx,
                   process_rx: response_rx}
    }

    ///////////////////////////////////////////////////////////////////////////
    pub fn request(&self, request: Request)
    {
        self.process_tx.send(request);
    }

    ///////////////////////////////////////////////////////////////////////////
    pub fn request_response(&self, request: Request) -> Response
    {
        self.process_tx.send(request);
        self.process_rx.recv()
    }
}    

    
