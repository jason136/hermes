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
    
                app.post('/upload/' + this.port + '/' + filename, (req, res) => {
                    console.log('post request recieved!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!');
                    var newpath = __dirname + '\\uploads\\' + filename;
                    console.log(newpath);
                    
                    req.on('data', (data) => {
                        this.buffer = Buffer.concat([this.buffer, data]);
                    });
    
                    console.log('body: ' + this.body);
                    req.on('end', () => {
                        fs.writeFile(newpath, this.buffer, "binary", () => {
                            res.end();
                            console.log(this.buffer);
                            console.log('File written to ' + newpath);
                        });
                    });

                    //req.pipe(fs.createWriteStream(filepath))
                    res.end();
                    console.log('File written to ' + newpath);
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
                        var filepath = line.substring(6).replaceAll('\"', '').replaceAll('\'', '').replaceAll('\\', '/');
                        var filename = filepath.split('/').pop();
                        app.get('/download/' + this.port + '/' + filename, (req, res) => {
                            res.download(filepath);
                            console.log('file sent ', filepath);
                        });
    
                        var filename = filepath.split('/').pop();
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