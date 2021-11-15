const Express = require('express');
const app = Express();

const fs = require('fs')
const WebSocket = require('ws');

const readline = require('readline');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
rl.setPrompt('');

function httpEncode(name) {
    return name.replaceAll(' ', '%20').replaceAll('(', '%28').replaceAll(')', '%29')
}

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
                this.buffer = Buffer.from('');
    
                app.post('/upload/' + this.port + '/' + httpEncode(filename), (req, res) => {
                    var newpath = __dirname + '\\uploads\\' + filename;
                    req.on('data', (data) => {
                        this.buffer = Buffer.concat([this.buffer, data]);
                    });
    
                    req.on('end', () => {
                        fs.writeFile(newpath, this.buffer, "binary", () => {
                            res.end();
                            console.log('File written to ' + newpath);
                        });
                    });
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
                        var seperated = line.substring(6).replaceAll('\'', '\"').split('\"');
                        var filepaths = [];
                        for (let x = 0; x < seperated.length; x++) {
                            if (seperated[x].trim() != '') {
                                filepaths.push(seperated[x]);
                            }
                        }
                        var files = [];
                        for (let x = 0; x < filepaths.length; x++) {
                            var filepath = filepaths[x].replaceAll('\\', '/');
                            var filename = filepath.split('/').pop();
                            console.log(filepath);
                            files.push(filepath);
                            app.get('/download/' + this.port + '/' + httpEncode(filename), (req, res) => {
                                let file = files.shift();
                                console.log(files.length);
                                res.download(file);
                                console.log('file sent ', file.split('/').pop());
                            });

                            var filename = filepath.split('/').pop();
                            socket.send('/file ' + filename);
                        }
                        break;
                    default:
                        socket.send(line);
                        break;
                }
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