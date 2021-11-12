const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
rl.setPrompt('message> ');

const WebSocket = require('ws');
const server = new WebSocket.Server({port: '8080'});

server.on('connection', socket => {
    console.log('socket is connected');

    socket.on('message', message => {
        console.log('message recieved: ', message.toString());
        socket.send(`Roger that! ${message}`);
        rl.prompt();
    });
    
    rl.prompt();

    rl.on('line', (line) => {
        if (line) socket.send(line);
        rl.prompt();
    }).on('close', () => {
        resolve(42);
    });
});




const Express = require('express');
const app = Express();
const router = require('./router.js');
const { resolve } = require('path');

app.use('/', router);
app.listen(3000);