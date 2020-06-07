const assert = require('assert');
const Net = require('net');
const Tls = require('tls');
const EventEmitter = require('events');
const {Executor} = require('../pkg/server');

let id = 1;

const getId = () => id++;

const serverData = {};

class Server extends EventEmitter {
  /**
   * @typedef {Object} ConstructorOptions
   * @param {Executor} executor
   * @param {Object} proto
   */

  /**
   * Creates an instance of Server.
   * @param {ConstructorOptions} options
   */
  constructor(options) {
    super();

    const executor = options && options.executor;
    assert(executor, '"executor" option is required');

    const id = getId();

    this._id = () => id;

    const proto = options.proto || {};

    serverData[id] = {
      master: executor,
      proto,
      middlewares: []
    };
  }

  free() {
    const id = this._id();
    serverData[id].master.free();
    serverData[id] = null;
  }

  listen(options) {
    const me = this;

    const execMdws = this._compose();

    const instData = serverData[this._id()];

    const timerIds = new Map();
    const parseTimeout = options.parseTimeout != null ? options.parseTimeout * 1000 : null;

    const onBeforeParse = instData.proto.onBeforeParse;
    const onAfterParse = instData.proto.onAfterParse;
    const onTknData = instData.proto.onTknData;

    function timeOutHandler(ctx) {
      ctx.socket && ctx.socket.emit('__parse_init__');
      me.emit('error', {
        error: 'Timeout',
        ctx
      });
    }

    instData.proto.onBeforeParse = function() {
      (typeof(onBeforeParse) == 'function') && onBeforeParse.call(this);
      // set timer
      (parseTimeout != null) && timerIds.set(this, setTimeout(timeOutHandler, parseTimeout, this));
    };
    instData.proto.onAfterParse = function() {
      // clear parseTimeout
      if (parseTimeout != null) {
        clearTimeout(timerIds.get(this));
        timerIds.delete(this);
      }
      (typeof(onAfterParse) == 'function') && onAfterParse.call(this);
      // exec middlewares
      execMdws(this);
    };

    const connectionListener = async (socket) => {

      const instData = serverData[me._id()];
      let executor = instData.master.clone_executor();

      socket.on('data', (data) => {
        try {
          executor.parse_data(data, instData.proto, 'socket', socket,
          instData.proto.onBeforeParse, instData.proto.onAfterParse, onTknData);
        } catch (err) {
          executor.parse_init();
          me.emit('error', {
            error: err,
            socket
          });
        }
      });

      socket.on('__parse_init__', () => {
        executor.parse_init();
      });

      socket.on('error', (err) => {
      });

      socket.on('close', () => {
        executor.free();
        executor = null;
      });
    };

    const createOptions = {};
    let module;
    if (options.tls === true) {
      module = Tls;
      options.key != null && (createOptions.key = options.key);
      options.cert != null && (createOptions.cert = options.cert);
      options.requestCert != null && (createOptions.requestCert = options.requestCert);
      options.ca != null && (createOptions.ca = options.ca);
    } else {
      module = Net;
    }

    const internal = module.createServer(createOptions, connectionListener);


    if (this.listenerCount('error') == 0) {
      this.on('error', this.onError);
    }
    internal.maxConnections = 100;
    internal.listen(options);
  }

  /**
   * Default error handler
   */
  onError(evt) {
    console.log(evt.error);
  }

  /**
   * Adds middleware.
   * @param {Function} middleware
   * @return {Server}
   */
  use(middleware) {
    if (typeof(middleware) != 'function') throw new Error('param must be function');
    const instData = serverData[this._id()];
    instData.middlewares.push(middleware);
    return this;
  }

  _compose() {
    const instData = serverData[this._id()];
    if (instData.middlewares.length == 0) throw new Error('Middlewares is not set');

    const mds = instData.middlewares.slice();
    let execIndex = 0;
    let context;

    async function next() {
      execIndex++;
      let fn = mds[execIndex];
      if (!fn) throw new Error('Next middlware does not exist');
      return fn(context, next);
    }

    return async (ctx) => {
      context = ctx;
      execIndex = -1;
      await next();
    };
  }
}

/**
 * @typedef {Object} ServerBuilderOptions
 * @property {String} regexp - regular expression text
 * @property {String} grammar - grammar text
 * @property {Object} [proto] - prototype for message context
 */

/**
 * Creates and return an instance of Server.
 * @param {ServerBuilderOptions} options
 * @return {Server}
 */
function build(options) {
  const regexp = options && options.regexp;
  assert(regexp, '"regexp" option must be set');
  const grammar = options && options.grammar;
  assert(grammar, '"grammar" option must be set');
  const proto = options && options.proto;

  const executor = Executor.build(regexp, grammar);

  const server = new Server({
    executor,
    proto,
  });

  return server;
}

exports.build = build;
