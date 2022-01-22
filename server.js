const app = require('express')();

const fs = require('fs')
const WebSocket = require('ws');

const readline = require('readline');

function httpEncode(name) {
    return name.replaceAll(' ', '%20').replaceAll('(', '%28').replaceAll(')', '%29')
}

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
rl.setPrompt('');

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
        this.server.on('connection', (socket) => {
            console.log('Client is connected on port ' + this.port);
            if (names.length == 1 || selecting) select();
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
                                this.sendOutput(`File written to ${newpath}`);
                            });
                        });
                    });

                    this.sendOutput('File upload command recieved, server is ready');
                    socket.send('/expt ' + filepath);
                }
                else this.sendOutput(message.toString(), true);
            });

            socket.on('close', socket => {
                console.log(`Client "${this.name}" disconnected`);
                let index = names.indexOf(this.name);
                names.splice(index, 1);
                servers.splice(index, 1);
                this.active = false;
                if (selecting || servers.length == 0) select(this);
            });

            rl.on('line', (line) => {
                if (!this.active) return;
                process.stdout.write('<<>>  ');
                line = line.replace('<<>>', '').replace('>><<', '');
                if (line.trim()) {
                    this.messageLog.push(line.trim());
                    if (this.messageLog.length == 1) return;
                    switch (line.substring(0, 5)) {
                        case '/file':
                            if (fs.existsSync(line.substring(6).replaceAll('"', '').trim())) {
                                var seperated = line.substring(6).replaceAll('\'', '\"').split('\"');
                                var files = {};
                                for (let x = 0; x < seperated.length; x++) {
                                    if (seperated[x].trim() != '') {
                                        let filename = seperated[x].replaceAll('\\', '/').split('/').pop();
                                        files[filename] = seperated[x];
                                        app.get('/download/' + this.port + '/' + httpEncode(filename), (req, res) => {
                                            res.download(files[filename]);
                                            this.sendOutput('File sent');
                                        });
                                        socket.send('/file ' + filename);
                                    }
                                }
                            }
                            else this.sendOutput('Directory is invalid');
                            break;
                        case '/dldr':
                            if (fs.existsSync(line.substring(6).replaceAll('"', '').trim())) {
                                fs.access(line.substring(6).replaceAll('"', '').trim(), (error) => {
                                    if (error) {
                                        this.sendOutput('Directory is invalid ', error)
                                    }
                                    else {
                                        this.directory = line.substring(6).replaceAll('"', '').trim();
                                        this.sendOutput(`Download directory changed to "${this.directory}`);
                                    }
                                });
                            }
                            else this.sendOutput('Directory is invalid');
                            break;
                        case '/name':
                            names[names.indexOf(this.name)] = line.substring(6);
                            this.name = line.substring(6);
                            this.sendOutput(`Client name changed to ${this.name}`);
                            break;
                        case '/exit':
                            this.active = false;
                            select();
                            break;
                        case '/clos':
                            socket.close();
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
        for (var x = 0; x < this.messageLog.length; x++) {
            console.log(this.messageLog[x]);
        }
        this.active = true;
        selecting = false;
    }
    sendOutput(input, msg=false) {
        if (this.active) {
            if (msg) console.log('\r>><<  ' + input);
            else console.log('\r' + input);
            process.stdout.write('<<>>  ');
            this.messageLog.push(input);
        }
    }
}

var selecting;

function select(to_delete) {
    if (to_delete) {
        socket_ports.push(to_delete.port);
        delete to_delete;
    }
    console.clear();
    selecting = true;
    
    console.log(`Listening on port ${http_port}`);
    console.log(`Ports available for websockets: ${socket_ports}\n`);
    if (names.length == 0) return;
    console.log('Connected clients:\n------------------------------');
    for (var x = 0; x < names.length; x++) {
        console.log(`${x + 1} - ${names[x]}`)
    }
    console.log('------------------------------\nChoose a client to connect to: ');
    rl.prompt();

    rl.on('line', (line) => {
        if (!selecting) return;
        if (line > 0 && line <= names.length) {
            servers[line - 1].activate();
            return;
        }
        else select();
    });
}

app.get('/checkin', (req, res) => {
    let port = socket_ports.shift();
    if (port) {
        names.push('client on ' + port);
        res.send('server online ' + port);
        servers.push(new websocketServer(port));
    }
    else res.send('server online ' + 'full');
});

// Use the following ports to establish websocket connections
var socket_ports = ['8080', '8081', '8082'];

// Listen for incoming http request on the following port
var http_port = 3000;

app.listen(http_port);
select();