const {build} = require('./server');

let ts = Date.now();
const server = build();
console.log(`server is built in ${Date.now() - ts} msec`);
server.use((ctx, next) => {
  ctx.connection.socket.write('result: ');
  ctx.connection.socket.write(ctx.result);
  ctx.connection.socket.write('\r\n');
})
.handler('errorConnection', (conn, err) => {
  conn.socket.write(`${err.toString()}\r\n`);
})
.listen({
  port: 5555
});
