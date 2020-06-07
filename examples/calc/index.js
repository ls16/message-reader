const {build} = require('./server');

let ts = Date.now();
const server = build();
console.log(`server is built in ${Date.now() - ts} msec`);
server.use((ctx, next) => {
  ctx.socket.write('result: ');
  ctx.socket.write(ctx.result);
  ctx.socket.write('\r\n');
})
.on('error', (evt) => {
  evt.socket.write(evt.error.toString());
  evt.socket.write('\r\n');
})
.listen({
  port: 5555
});
