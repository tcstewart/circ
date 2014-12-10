///////////////////////////////////////////////////////////////////////////////

use time::Timespec;

use irc::data::message::Message;

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
    pub messages: Vec<(Timespec, Message)>
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
    pub fn add(&mut self, msg: (Timespec, Message))
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
