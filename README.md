# Hermes
Hermes is a websocket and http protocol powered messaging and file transfer app. It is split into an app each for the client and the server. 
Both the client and the server are used through console windows, as of now there is no fancy gui.
An instance of the server can support multiple connections to various clients. A client, however, can only be connected to one server at a time. \
\
The [sever side app](server.js) is written in javascript and can be run with Node.js. \
The [client side app](src/main.rs) is written in rust and must be compiled into a platform specific executable before it is used. 

# How to
Most content typed into the console window both apps create will be automatically sent to the corresponding destination; 
however, some commands (prefixed by '/') will carry out commands instead and will not be sent. \
This is to say if you're trying to send the message "/file hello world", the message will not be sent and an error may be shown. 
## Server
To get started, run the [server.js](server.js) script with Node.js and allow it to listen for http requests on a predefined port (the default is port 3000). \
In addition to this port, the app needs to be allowed to bind to other ports for websocket connections to be established (the default is ports 8080, 8081, 8082). \
The http server will always be bound to the same port while the websocket ports will be "used up" as more connections are established, 
for this reason it is reccomended to provide multiple ports for websocket connections. 
Both of these values can be found at the very bottom of the file and can be swapped out before running the script. 
```javascript
var ports = ['8080', '8081', '8082'];
app.listen(3000);
```
Please note that traffic will only come through if the ports are properly port forwarded on the server's network. \
When the server is running without any established connections, it will display the available ports for websocket connections. 
When an websocket handshake is completed the client will show up on the server's list of connected clients with the default name (client on ****). \
A client is chosen to interact with by typing its corresponding number and hitting enter.\
The display name can be changed with one of the server specific commands: \
- ``/name `` \
Whatever succedes this command will become the new name that is displayed for the connected client \
Example: ``/name John Doe``
- ``/exit `` \
Backs out of the client interaction page and displays the client selection page. Note: this does not close the connection to the client \
- ``/close `` \
Sends a close handshake message to the client and closes the connection. \
## Client
When the client executable is run, the user will be immediately prompted to provide an ip address where a server instance can be found. 
When a connection is established, the client will be able to send and recieve messages and files to and from the server. \
The client does not have any unique commands. 
## Shared commands
The following commands can be used on both the client and server apps with the same effect. 
- ``/file `` \
This command takes the succeeding path to a file on the local computer and sends it to the other computer. 
This command can be followed by multiple file paths, each surrounded by "", to send multiple files with a single command. \
Examples: ``/file C:/cool_story.txt`` or ``/file /cool_story.txt`` or ``/file "C:\stories\cool_story.txt" "C:\pictures\cool_picture.png"``
- ``/dldr `` \
Standing for download directory, this command takes the succeeding path to a folder on the local computer and sets it as the new location to write all incoming files to. 
Examples: ``/dldr C:\Users\John\Downloads`` or ``/dldr /home/John/Downloads``/
note both commands are followed by a space and then the parameters. 
## Disclaimer 
please note that data sent over the internet with this app is not encrypted, so please don't send any sensitive info