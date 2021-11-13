const Express = require('express');
const router = Express.Router();

router.get('/checkin', (req, res) => {

    res.send('server online');

});

router.post('/', (req, res) => {

    res.send('post recieved!');

});

module.exports = router;