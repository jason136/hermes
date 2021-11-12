const Express = require('express');
const router = Express.Router();

router.get('/', (req, res) => {

    res.send('Hello World... from router!');

});

router.post('/', (req, res) => {

    res.send('post recieved!');

});

module.exports = router;