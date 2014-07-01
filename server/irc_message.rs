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


extern crate time;

use time::Timespec;

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show,Clone)]
pub struct Message
{
    pub time: Timespec,
    pub prefix: Option<String>,
    pub command: String,
    pub parameters: Vec<String>,
    pub trailing: Option<String>
}


static EOM: &'static str = "\r\n";
static PROGRAM: &'static str = "circ";

///////////////////////////////////////////////////////////////////////////////
impl Message
{
    ///////////////////////////////////////////////////////////////////////////
    pub fn new() -> Message
    {
        Message{time      : time::get_time(),
                prefix    : None,
                command   : String::new(),
                parameters: Vec::new(),
                trailing  : None}
    }

    ///////////////////////////////////////////////////////////////////////////
    pub fn parse(buffer: &[u8]) -> Message
    {
        let line = match ::std::str::from_utf8(buffer)
        {
            Some(s) => s,
            None => fail!("No message recieved")
        };
        
        let mut msg = Message::new();

        let mut tokens = line.split(' ');
        
        let mut token = tokens.next().unwrap();
        
        let (c, s) = token.slice_shift_char();
        
        if c == Some(':')
        {
            msg.prefix = Some(from_str(s).unwrap());
            token = tokens.next().unwrap();
        }

        msg.command.push_str(token);
        
        let mut trailing = String::new();
        let mut is_trailing = false;
        for token in tokens
        {
            let (c, s) = token.slice_shift_char();

            if is_trailing
            {
                trailing.push_str(" ");
                trailing.push_str(token);
            }
            else if c == Some(':')
            {
                trailing.push_str(s);
                
                is_trailing = true;
            }
            else
            {
                msg.parameters.push(from_str(token).unwrap());
            }
        }
        
        if is_trailing {msg.trailing = Some(trailing);}
        
        msg 
    }

    // Building commands to send
    pub fn nick(nick: &String) -> String
    {
        format!("NICK {}{}", nick, EOM)
    }
    
    pub fn user(name: &String) -> String
    {
    format!("USER {} 8 * :{}{}", PROGRAM, name, EOM)
    }
    
    pub fn join(channel: &String) -> String
    {
        format!("JOIN {}{}", channel, EOM)
    }
    
    pub fn part(channel: &String) -> String
    {
        format!("PART {}{}", channel, EOM)
    }
    
    pub fn pong(data: &String) -> String
    {
        format!("PONG :{}{}", data, EOM)
    }

    pub fn msg(dest: &String, data: &String) -> String
    {
        format!("PRIVMSG {} :{}{}", dest, data, EOM)
    }

    pub fn quit() -> String
    {
        format!("QUIT{}", EOM)
    }
    
}


