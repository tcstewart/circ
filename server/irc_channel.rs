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

use irc_message::Message;

///////////////////////////////////////////////////////////////////////////////
#[deriving(Show)]
pub struct User
{
    pub name   : String,
    pub client : String,
    pub address: String
}

/*
impl User
{
    pub fn parse(data: String) -> User
    {
        User{name: String::new(), client: String::new(), address: String::new()}
    }
}
*/
///////////////////////////////////////////////////////////////////////////////
#[deriving(Show)]
pub struct Channel
{
    pub name: String,
    pub topic: String,
    pub users: Vec<User>,
    pub messages: Vec<Message>
}

///////////////////////////////////////////////////////////////////////////////
impl Channel
{
    ///////////////////////////////////////////////////////////////////////////
    pub fn new(name: &str) -> Channel
    {
        Channel{name: name.to_string(), topic: String::new(), users: Vec::new(), messages: Vec::new()}
    }

    ///////////////////////////////////////////////////////////////////////////
    pub fn set_topic(&mut self, topic: &str)
    {
        self.topic = topic.to_string();
    }

    ///////////////////////////////////////////////////////////////////////////
    pub fn add(&mut self, msg: Message)
    {
        self.messages.push(msg);
        // also write it to a log file at this point?
    }

    ///////////////////////////////////////////////////////////////////////////
    //pub fn unread_msgs(&mut self) 
    //pub fn recent_msgs(&mut self, seconds: u32)
    //pub fn last_msgs(&mut self, num: u32)
    //
    ///////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self)
    {
        self.messages.clear();
    }

}
