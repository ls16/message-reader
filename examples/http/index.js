const {build} = require('./server');

let ts = Date.now();
const server = build();
console.log(`server is built in ${Date.now() - ts} msec`);
server.use((ctx, next) => {
  //console.log('request:', ctx.request);
  let respText = 'HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n';
  ctx.socket.write(respText);
})
.on('error', (evt) => {
  console.error('Ошибка:', evt.error);
  let respText = 'HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n';
  evt.socket.write(respText);
})
.listen({
  port: 5555
});
