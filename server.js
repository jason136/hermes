const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
rl.setPrompt('');

const WebSocket = require('ws');
const server = new WebSocket.Server({port: '8080'});

server.on('connection', socket => {
    console.log('Client is connected');

    socket.on('message', message => {
        console.log('Message Recieved: ', message.toString());
        //socket.send(`Roger that! ${message}`);
        rl.prompt();
    });

    socket.on('close', socket => {
        console.log('Client disconnected')
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

app.get('/checkin', (req, res) => {

    res.send('server online');

});

app.get('/download', (req, res) => res.download('./hayasaka.jpg'));

app.post('/', (req, res) => {

    res.send('post recieved!');

});

app.listen(3000);