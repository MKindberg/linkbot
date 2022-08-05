# linkbot

An annoying chatbot for discord, it will reply to every message from a user with one of a pre-determined set of messages.

## Commands
* `!set <user>|<message1>|<message2>|<message3>|...` - Start replying with one of the messages when <user> says something.
* `!set <user> <message>` - Start replying to messages from <user> with <message>
* `!unset <user>` - unsets all messages for <user>
* `!add <user>|<message>` Add <message> to the list of replies for <user>
* `!add <user> <message>` Same as above if <user> doesn't contain spaces.

## Start
To run the bot set the enviroment variable `DISCORD_TOKEN` to the bots discord token and `USER_REPLY_FILE` to a file that will be used as a database.
