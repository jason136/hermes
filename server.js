const Express = require('express');
const app = Express();

const fs = require('fs')
const WebSocket = require('ws');

const readline = require('readline');

function httpEncode(name) {
    return name.replaceAll(' ', '%20').replaceAll('(', '%28').replaceAll(')', '%29')
}

let rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
rl.setPrompt('');

var ports = ['8080', '8080'];
var servers = [];
var names = [];

class websocketServer {
    constructor(port) {
        this.port = port,
        this.name = 'client on ' + this.port;
        this.active = false;
        this.messageLog = [];
        this.server = new WebSocket.Server({ port: port });
        this.directory = './uploads/';
        this.server.on('connection', socket => {
            console.log('Client is connected on port ' + this.port);
            if (names.length = 1) select();
            socket.on('message', message => {
                if (message.toString().substring(0, 5) == '/file' && message.toString().trim().length > 8) {
                    if (this.directory == './uploads/') !fs.existsSync('./uploads/') && fs.mkdirSync('./uploads/', { recursive: true });
                    var filepath = message.toString().substring(6).replaceAll('\"', '').replaceAll('\'', '').replaceAll('\\', '/');
                    var filename = filepath.split('/').pop();

                    app.post('/upload/' + this.port + '/' + httpEncode(filename), (req, res) => {
                        var newpath;
                        this.directory == './uploads/' ? newpath = __dirname + '/uploads/' + filename : newpath = this.directory + '/' + filename;
                        var buffer = Buffer.from('');

                        req.on('data', (data) => {
                            buffer = Buffer.concat([buffer, data]);
                        });

                        req.on('end', () => {
                            fs.writeFile(newpath, buffer, "binary", () => {
                                res.end();
                                this.sendOutput('File written to ' + newpath);
                            });
                        });
                    });

                    this.sendOutput('File upload command recieved, server is ready');
                    socket.send('/expt ' + filepath);
                }
                else {
                    this.sendOutput('Message Recieved: ' + message.toString());
                }
            });

            socket.on('close', socket => {
                this.sendOutput('Client disconnected');
            });

            rl.on('line', (line) => {
                if (line.trim() && this.active) {
                    switch (line.substring(0, 6)) {
                        case '/file ':
                            var seperated = line.substring(6).replaceAll('\'', '\"').split('\"');
                            var files = {};
                            for (let x = 0; x < seperated.length; x++) {
                                if (seperated[x].trim() != '') {
                                    let filename = seperated[x].replaceAll('\\', '/').split('/').pop();
                                    files[filename] = seperated[x];
                                    app.get('/download/' + this.port + '/' + httpEncode(filename), (req, res) => {
                                        res.download(files[filename]);
                                        this.sendOutput('file sent ', filename);
                                    });
                                    socket.send('/file ' + filename);
                                }
                            }
                            break;
                        case '/direc':
                            if (fs.existsSync(line.substring(6).replaceAll('"', '').trim())) {
                                fs.access(line.substring(6).replaceAll('"', '').trim(), (error) => {
                                    if (error) {
                                        this.sendOutput('Directory is invalid ', error)
                                    }
                                    else {
                                        this.directory = line.substring(6).replaceAll('"', '').trim();
                                        this.sendOutput(`Download directory changed to "${this.directory}"`);
                                    }
                                });
                            }
                            else {
                                this.sendOutput('Directory is invalid');
                            }
                            break;
                        case '/name ':
                            names[names.indexOf(this.name)] = line.substring(6);
                            this.name = line.substring(6);
                            this.sendOutput(`Client name changed to ${this.name}`);
                            break;
                        case '/exit ':
                            this.active = false;
                            select();
                            break;
                        default:
                            socket.send(line);
                            break;
                    }
                }
            });
        });
    }
    activate() {
        console.clear();
        this.active = true;

        for (var x = 0; x < this.messageLog.length; x++) console.log(this.messageLog[x]);

        this.messageLog = [];
    }
    sendOutput(input) {
        if (this.active) console.log(input);
        else this.messageLog.push(input);
    }
}

function select() {

    console.clear();
    var limiter = false;

    console.log('Connected clients:\n------------------------------');
    for (var x = 0; x < names.length; x++) {
        console.log(`${x + 1} - ${names[x]}`)
    }
    console.log('------------------------------\nChoose a client to connect to: ');
    rl.prompt();
    
    rl.on('line', function handler(line) {
        if (limiter) return;
        else limiter = true;
        console.log('rl shenanegans')
        if (line > 0 && line <= names.length) {
            servers[line - 1].activate();
            delete rl;
            return;
        }
    });
}

app.get('/checkin', (req, res) => {

    let port = ports.shift();
    servers.push(new websocketServer(port));
    names.push('client on ' + port);
    res.send('server online ' + port);

});

app.listen(3000);
console.log('Listening on port 3000')
console.log(`Ports available for websockets: ${ports}`)