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

class websocketServer {
    constructor(port) {
        this.port = port,
        this.server = new WebSocket.Server({ port: port });
        this.directory = './uploads/';
        this.server.on('connection', socket => {
            console.log('Client is connected on port ' + this.port);

            socket.on('message', message => {
                if (message.toString().substring(0, 5) == '/file' && message.toString().trim().length > 8) {
                    if (this.directory == './uploads/') !fs.existsSync('./uploads/') && fs.mkdirSync('./uploads/', { recursive: true });
                    var filepath = message.toString().substring(6).replaceAll('\"', '').replaceAll('\'', '').replaceAll('\\', '/');
                    var filename = filepath.split('/').pop();

                    app.post('/upload/' + this.port + '/' + httpEncode(filename), (req, res) => {
                        var newpath;
                        this.directory == './uploads/' ? newpath = __dirname + '\\uploads\\' + filename : newpath = this.directory + '\\' + filename;
                        var buffer = Buffer.from('');

                        req.on('data', (data) => {
                            buffer = Buffer.concat([buffer, data]);
                        });

                        req.on('end', () => {
                            fs.writeFile(newpath, buffer, "binary", () => {
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
                console.log('Client disconnected');
            });

            rl.prompt();
            rl.on('line', (line) => {
                if (line.trim()) {
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
                                        console.log('file sent ', filename);
                                    });
                                    socket.send('/file ' + filename);
                                }
                            }
                            break;
                        case '/direc':
                            if (fs.existsSync(line.substring(6).replaceAll('"', '').trim())) {
                                fs.access(line.substring(6).replaceAll('"', '').trim(), (error) => {
                                    if (error) {
                                        console.log('Directory is invalid asdasdasd', error)
                                    }
                                    else {
                                        this.directory = line.substring(6).replaceAll('"', '').trim();
                                        console.log(`Download directory changed to "${this.directory}"`);
                                    }
                                });
                            }
                            else {
                                console.log('Directory is invalid');
                            }
                            break;
                        default:
                            socket.send(line);
                            break;
                    }
                }
                rl.prompt();
            });
        });
    }
}

var ports = ['8080', '8080'];
var servers = [];

app.get('/checkin', (req, res) => {

    let port = ports.shift();
    servers.push(new websocketServer(port));
    res.send('server online ' + port);

});

app.listen(3000);