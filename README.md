circ
====

A command line interface to chat on IRC.

The reason for this tool was primarily to learn rust, but I had a secondary goal of trying to make a tool I could use to integrate IRC more into my command line to hopefully allow me to remove one more window I have to have open at work.

How I use
====

In the background I have the deamon process running
 > circd -n tcstewart -r TCStewart irc.mozilla.org

In my .bashrc:
I join channels I am interested in
 > circ -c \#rust -j

Setup status to show with the prompt
 > PROMPT_COMMAND="circ -c \#rust -s"
 
Then above my prompt I will see the status
 > rust has 5 new messages

 > prompt> 
 
To send a message:
 > circ -c \#rust -m Can anyone explain to me about borrowing and boxes and lifetime?
 
To show the unread messages:
 > circ -c \#rust -u


Limitations/Future enhancements
====
* Currently only one server is supported.
* Unix Socket used between circ and circd assumes a folder ${HOME}/.circd exists
* Private messages to a user (kind of works, but incoming messages aren't handled well)
* Would like to get ride of the use of the # in the channel name as it has to be escaped (I use aliases to get around having to type channel names)
* No logging
* Once I have logging, would like to have searching of history
* Ability to add search terms for more important messages (separate status output when your nick appears in messages)
* etc
