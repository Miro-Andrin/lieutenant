# Lieutenant: 
> A command creation and dispatching system for Feather


# How it works
The feather Minecraft server originally based their command system on something 
that was a rewrite of mojangs [brigadier](https://github.com/Mojang/brigadier) 
in rust. The current implementation works differently.

Every time a plugin is registered with the server it submits a list of 
regex strings, and command id's. Using this list of regex the server can
determine what commands could potentially parse the input. Then when someone
writes something in the chat/terminal, the server looks at all of its regular
expressions and if one or more of them roughly matches the input, then the 
server notifies the plugin what commands were triggered and gives the
command the message that the user wrote in chat. The input string is then
parsed and acted upon inside of the plugin. If multiple command regex matched
on the input, then it just tries one command after until one successfully runs. 

Note: 
   The server does not necessarily match the whole regex before it decides
   to trigger the command. If lets say only one command starts with '/z', namely
   '/zipit <player> you are too loud', then the server might trigger for the
   chat message '/zebra' because it sees /z and concludes that only one command
   starts with '/z'. The command has to attempt parsing anyways, so doing this 
   early termination is good. 



