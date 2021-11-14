const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
rl.setPrompt('');

const Express = require('express');
const app = Express();

const fs = require('fs')

const WebSocket = require('ws');
const { request } = require('http');

function websocketServer(port) {
    this.port = port, 
    this.server = new WebSocket.Server({port: port});    
    this.server.on('connection', socket => {
        console.log('Client is connected');
    
        socket.on('message', message => {
            if (message.toString().substring(0, 5) == '/file') {
                !fs.existsSync(`./uploads/`) && fs.mkdirSync(`./uploads/`, { recursive: true });
                var filepath = message.toString().substring(6).replaceAll('\"', '').replaceAll('\'', '').replaceAll('\\', '/');
                var filename = filepath.split('/').pop();
                console.log(filepath)
    
                app.post('/upload', (req, res) => {
                    console.log('post request recieved!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!');
                    var body;
                    filepath = __dirname + '\\uploads\\' + filename;
                    console.log(filepath);
                    
                    req.on('data', (data) => {
                        body += data;
                    });
    
                    console.log('body: ' + body);
                    req.on('end', () => {
                        fs.appendFile(filepath, body, () => {
                            res.end();
                        });
                    });
    
                    console.log('File uploaded ' + filename);
                });
                
                console.log('File upload command recieved, server is ready');
                socket.send('/expt ' + filepath);
            }
            else {
                console.log('Message Recieved:', message.toString());
            }
            rl.prompt();
        });
    
        socket.on('close', socket => {
            console.log('Client disconnected')
        });
    
        rl.prompt();
        rl.on('line', (line) => {
            if (line.trim()) {
                switch (line.substring(0, 5)) {
                    case '/file':
                        fullpath = line.substring(6).replaceAll('\"', '').replaceAll('\'', '').replaceAll('\\', '/');
    
                        app.get('/download', (req, res) => {
                            res.download(fullpath);
                            console.log('file sent ', fullpath);
                        });
    
                        var filename = fullpath.split('/').pop();
                        line = ('/file ' + filename);
                        console.log(line);
                        break;
                        
                }
                socket.send(line);
            }
            rl.prompt();
        })
    });   
}

var servers = [];
const port = '8080';

app.get('/checkin', (req, res) => {

    servers.push(new websocketServer(port));
    res.send('server online ' + port);

});

app.listen(3000);