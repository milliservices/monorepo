const http = require('http')

http.createServer((req, res) => {
  if (req.url === '/rust-final') {
    let data = ''
    req.on('data', c => (data += c.toString()))
    req.on('end', () => {
      console.log('J', req.headers.authentication);
      console.log('Final content here: ', data);
      res.write(`Return to sender: ${data}`)
      res.end()
    })
  }
}).listen(3001)
