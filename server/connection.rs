///////////////////////////////////////////////////////////////////////////////
use std::collections::HashMap;
use std::collections::hash_map::{Occupied,Vacant};
use std::string::String;
use std::sync::Arc;

use time;
use time::Timespec;

use circ_comms;
use circ_comms::{Request, Response};
use irc_channel;

use irc::data::{Config, Message};
use irc::data::Command::{JOIN, PART, PRIVMSG, QUIT};
use irc::server::{IrcServer, Server, NetIrcServer};
use irc::server::utils::Wrapper;




///////////////////////////////////////////////////////////////////////////////
pub struct Connection
{
    process_tx: Sender<Request>,
    process_rx: Receiver<Response>
}


///////////////////////////////////////////////////////////////////////////////
fn rx_task(server: Arc<NetIrcServer>,
           tx: Sender<(Timespec, Message)>)
{
    spawn(move ||
          {
              for message in server.iter()
              {
                  debug!("{}", message.into_string());
                  tx.send((time::get_time(), message));
              }
          });
}

///////////////////////////////////////////////////////////////////////////////
fn set_topic(channels: &mut HashMap<String, irc_channel::Channel>, msg: Message)
{
    let name = msg.args[0].clone();
    let topic = match msg.suffix
       {
           Some(t) => t,
           None    => "No topic provided".to_string()
       };

    match channels.entry(name.clone())
    {
        Vacant(entry) =>
        {
            let mut c = irc_channel::Channel::new(name.as_slice());
            c.set_topic(topic.as_slice());
            entry.set(c);
        },
        Occupied(mut entry) => entry.get_mut().set_topic(topic.as_slice())
    };
}
///////////////////////////////////////////////////////////////////////////////
fn add_message(channels: &mut HashMap<String, irc_channel::Channel>,
               msg: (Timespec, Message))
{
    let name = msg.1.args[0].clone();
    if name == "AUTH".to_string() { return (); }

    match channels.entry(name.clone())
    {
        Vacant(entry) =>
        {
            let mut c = irc_channel::Channel::new(name.as_slice());
            c.add(msg.clone());
            entry.set(c);
        },
        Occupied(mut entry) => entry.get_mut().add(msg.clone())
    };
}
 
///////////////////////////////////////////////////////////////////////////////
fn get_channels(channels: &HashMap<String, irc_channel::Channel>) -> Response
{
    let mut names : Vec<String> = Vec::new();

    for name in channels.keys()
    {
        names.push(name.clone());
    }

    circ_comms::Response::Channels(names)
}

///////////////////////////////////////////////////////////////////////////////
fn get_messages(channels: &mut HashMap<String, irc_channel::Channel>,
                name: &str) -> Response
{
    let channel = channels.get_mut(&name.to_string());

    match channel
    {
        Some(c) =>
        {
            let mut r = Vec::new();
            for m in c.messages.iter()
            {
                // TODO: I need to understand the borrowing much better...
                let user = match m.1.prefix.clone()
                    {
                        Some(p) => p,
                        None    => "Unknown User".to_string()
                    };
                let msg = match m.1.suffix.clone()
                    {
                        Some(m) => m,
                        None    => "No message".to_string()
                    };

                r.push(circ_comms::Message::new(m.0,
                                                user.as_slice(),
                                                msg.as_slice()));
            }
            c.clear();
            circ_comms::Response::Messages(r)
        },
        None    => circ_comms::Response::Error(format!("Unknown channel {}", name))
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
    
    circ_comms::Response::Status(statuses)
    
}
///////////////////////////////////////////////////////////////////////////////
fn process_task(rx: Receiver<(Timespec, Message)>,
                tx: Arc<NetIrcServer>,
                response_tx: Sender<Response>, 
                request_rx: Receiver<Request>)
{
    spawn(move ||
          {
              let mut channels = HashMap::new();
              let server = Wrapper::new(&*tx);

              server.identify().unwrap();
              loop
              {
                  select!((time, msg) = rx.recv() =>
                          match msg.command.as_slice()
                          {
                              "ERROR"   => {println!("Error... {}", msg);},
                              "TOPIC"   => set_topic(&mut channels, msg),
                              "PRIVMSG"|"NOTICE" => add_message(&mut channels, (time, msg)),
                              _         => () //println!("{}", msg)
                          },

                          request = request_rx.recv() =>
                          match request
                          {
                              circ_comms::Request::ListChannels =>
                                  response_tx.send(get_channels(&channels)),
                              circ_comms::Request::GetStatus =>
                                  response_tx.send(get_status(&channels)),
                              circ_comms::Request::GetMessages(channel) =>
                                  response_tx.send(get_messages(&mut channels,
                                                                channel.as_slice())),
                              circ_comms::Request::GetUsers(_) =>
                                  response_tx.send(circ_comms::Response::Users(Vec::new())),
                              circ_comms::Request::Join(channel) =>
                                  server.send(JOIN(channel.as_slice(), None)).unwrap(),
                              circ_comms::Request::Part(channel) =>
                                  server.send(PART(channel.as_slice(), None)).unwrap(),
                              circ_comms::Request::SendMessage(channel, msg) =>
                                  server.send(PRIVMSG(channel.as_slice(), msg.as_slice())).unwrap(),
                              circ_comms::Request::Quit =>
                                  { server.send(QUIT(None)).unwrap(); break }
                          });
              }
          });
}

///////////////////////////////////////////////////////////////////////////////
impl Connection
{
    ///////////////////////////////////////////////////////////////////////////
    pub fn new(config: Config) -> Connection
    {
        let irc_server = Arc::new(IrcServer::from_config(config).unwrap());

        // channel to handle communication with task receiving messages
        // from the irc server
        let (incoming_msg_tx, incoming_msg_rx) = channel();

        // duplex to handle communication with process task
        let ((request_tx, request_rx), (response_tx, response_rx)) = (channel(), channel());

        // Start up the task to receive messages from the irc server
        rx_task(irc_server.clone(), incoming_msg_tx);

        process_task(incoming_msg_rx, irc_server, response_tx, request_rx);
        
        
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

    
