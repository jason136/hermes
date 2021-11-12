const WebSocket = require('ws');
const server = new WebSocket.Server({port: '8080'});

server.on('connection', socket => {
    console.log('socket is connected');

    socket.on('message', message => {
        console.log('message recieved: ', message.toString());
        socket.send(`Roger that! ${message}`);
    });
});


const Express = require('express');
const app = Express();
const router = require('./router.js');

app.use('/', router);
app.listen(3000);