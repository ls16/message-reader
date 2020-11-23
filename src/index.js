const assert = require('assert');
const Net = require('net');
const Tls = require('tls');
const {Executor, hash} = require('../pkg/server');

const INIT = 0;
const LISTENING_START = 1;
const LISTENING = 2;
const CONNECTING = 3;
const CONNECTED = 4;
const CLOSING = 5;
const CLOSED = 6;

let id = 1;

const getId = () => id++;

const instancesData = {};

function compose(id) {
  const instData = instancesData[id];
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

function execHandler(handler, self, params) {
  try {
    return handler.apply(self, params);
  }
  catch (err) {}
}

function createConnectionListener(self, type, id, options, sock) {
  const execMiddlewares = compose(id);
  const instData = instancesData[id];
  const timerIds = new Map();
  const parseTimeout = options.parseTimeout != null ? options.parseTimeout * 1000 : null;

  function timeOutHandler(ctx) {
    ctx.connection.socket && ctx.connection.socket.destroy(new Error('Timeout'));
  }

  instData.execOptions.forEach((opt) => {
    const onBeforeParse = opt.proto.onBeforeParse;
    const onAfterParse = opt.proto.onAfterParse;

    opt.onBeforeParse = function() {
      (typeof(onBeforeParse) == 'function') && onBeforeParse.call(this);
      // set timer
      (parseTimeout != null) && timerIds.set(this, setTimeout(timeOutHandler, parseTimeout, this));
    };
    opt.onAfterParse = function() {
      // clear parseTimeout
      if (parseTimeout != null) {
        clearTimeout(timerIds.get(this));
        timerIds.delete(this);
      }
      (typeof(onAfterParse) == 'function') && onAfterParse.call(this);
      // exec middlewares
      execMiddlewares(this);

      const breakParse = instData.breakParse == true ? true : false;
      instData.breakParse = false;
      return breakParse;
    };
  });

  return (socket) => {
    let optionsIndex = 0;
    let executor;

    function parseInit() {
      executor.parse_init();
    }

    function parseData(data) {
      try {
        executor.parse_data(data, instData.execOptions[optionsIndex].proto, 'connection', connection,
        instData.execOptions[optionsIndex].onBeforeParse,
        instData.execOptions[optionsIndex].onAfterParse,
        instData.execOptions[optionsIndex].proto.onTknData);
      } catch (err) {
        executor.parse_init();
        execHandler(instData.handlers.errorConnection, connection, [connection, err]);
      }
    }

    function setOptions(index) {
      if (index < 0 || index >= instData.execOptions.length) return;
      const curExecutor = executor;
      optionsIndex = index;
      executor = type == 'server'
        ? instData.execOptions[optionsIndex].master.clone_executor()
        : instData.execOptions[optionsIndex].master;
      instData.breakParse = true;

      process.nextTick(() => {
        const data = curExecutor && curExecutor.has_data() ? curExecutor.data() : null;
        if (data != null) parseData(data);
      });
    }

    setOptions(0);
    if (type == 'client') socket = sock;

    let connection = {
      socket,
      parseInit,
      setOptions
    };

    if (instData.handlers.connection != null) {
      const result = execHandler(instData.handlers.connection, null, [socket]);
      if (result != null) {
        connection = {
          ...result,
          socket,
          parseInit,
          setOptions
        };
      }
    }

    socket.on('data', (data) => {
      parseData(data);
    });
  
    socket.on('error', (err) => {
      execHandler(instData.handlers.errorConnection, connection, [connection, err]);
    });
  
    socket.on('close', (hadError) => {
      executor.free();
      executor = null;
      if (instData.handlers.closeConnection != null) {
        execHandler(instData.handlers.closeConnection, connection, [connection, hadError]);
      }
    });
  };
}

class Base {
  /**
   * @typedef {Object} ConstructorOptions
   * @param {Executor} executor
   * @param {Object} proto
   */

  /**
   * Creates an instance.
   * @param {ConstructorOptions | Array<ConstructorOptions>} options
   */
  constructor(options) {
    const execOptions = [];
    if (Array.isArray(options)) {
      if (options.length == 0) throw new Error('Options length must not be zero');
      options.forEach((opt) => {
        const executor = opt.executor;
        assert(executor, '"executor" option is required');
        const proto = opt.proto || {};
        execOptions.push({
          master: executor,
          proto
        })
      });
    } else {
      const executor = options && options.executor;
      assert(executor, '"executor" option is required');
      const proto = options.proto || {};
      execOptions.push({
        master: executor,
        proto
      })
    }

    const id = getId();

    this._id = () => id;

    instancesData[id] = {
      execOptions,
      middlewares: [],
      handlers: {
        errorConnection: this.onError
      },
      state: INIT,
      data: {},
      breakParse: false
    };
  }

  /**
   * Clones an instance.
   */
  clone() {
    const id = this._id();
    const execOptions = instancesData[id].execOptions;
    const clonedExecOptions = execOptions.map((option) => {
      return {
        executor: option.master.clone_executor(),
        proto: option.proto
      };
    });

    return new this.constructor(clonedExecOptions);
  }

  /**
   * @inner
   */
  free() {
    const id = this._id();
    const execOptions = instancesData[id].execOptions;
    execOptions.forEach((opt) => {
      opt.master && opt.ptr && opt.master.free();
    });
    instancesData[id].execOptions = [];
    delete instancesData[id];
  }

  close() {
    throw new Error('Not implemented');
  }

  /**
   * Default error handler
   */
  onError(conn, err) {
    console.log(err.toString());
  }

  /**
   * Sets event handler
   * @param {String} name
   * @param {Function} cb
   */
  handler(name, cb) {
    const instData = instancesData[this._id()];
    let handlerNames = [
      'connection',
      'errorConnection',
      'closeConnection'
    ];
    if (this instanceof Server) {
      handlerNames = handlerNames.concat([
        'listening',
        'error',
        'close'
      ]);
    }
    if (handlerNames.indexOf(name) == -1) {
      throw new Error(`Invalid handler name ${name}`);
    }

    instData.handlers[name] = cb;

    return this;
  }

  /**
   * Adds middleware.
   * @param {Function} middleware
   * @return {Server}
   */
  use(middleware) {
    if (typeof(middleware) != 'function') throw new Error('param must be function');
    const instData = instancesData[this._id()];
    instData.middlewares.push(middleware);
    return this;
  }
}

function changeServerState(id, newState) {
  const instData = instancesData[id];
  const errMessage = 'Can\'t change state';
  switch (newState) {
    case LISTENING_START:
      if (instData.state != INIT) throw new Error(errMessage);
      break;
    case LISTENING:
      if (instData.state != LISTENING_START) throw new Error(errMessage);
      break;
    case CLOSING:
      if (instData.state != LISTENING) throw new Error(errMessage);
      break;
    case CLOSED:
      if (instData.state != CLOSING) throw new Error(errMessage);
      break;
  }
  instData.state = newState;
}

class Server extends Base {
  listen(options) {
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

    const type = 'server';
    const id = this._id();
    const instData = instancesData[id];
    const connectionListener = createConnectionListener(this, type, id, options);
    const internal = module.createServer(createOptions, connectionListener);
    internal.maxConnections = 100;

    changeServerState(id, LISTENING_START);

    internal.listen(options, () => {
      changeServerState(id, LISTENING);
      instData.data.internal = internal;
      if (instData.handlers.listening != null) {
        execHandler(instData.handlers.listening, this, []);
      }
    });
    internal.on('close', () => {
      changeServerState(id, CLOSED);
      this.free();
      instData.data.internal = null;
      if (instData.handlers.close != null) {
        execHandler(instData.handlers.close, this, []);
      }
    });
  }

  close() {
    const id = this._id();
    changeServerState(id, CLOSING);
    const instData = instancesData[id];
    const internal = instData.data.internal;
    internal.close();
  }
}

function changeClientState(id, newState) {
  const instData = instancesData[id];
  const errMessage = 'Can\'t change state';
  switch (newState) {
    case CONNECTING:
      if (instData.state != INIT) throw new Error(errMessage);
      break;
    case CONNECTED:
      if (instData.state != CONNECTING) throw new Error(errMessage);
      break;
    case CLOSING:
      if (instData.state != CONNECTED) throw new Error(errMessage);
      break;
    case CLOSED:
      if (instData.state != CONNECTED && instData.state != CLOSING) throw new Error(errMessage);
      break;
  }
  instData.state = newState;
}

class Client extends Base {
  connect(options) {
    const module = options.tls === true ? Tls : Net;
    const type = 'client';
    const id = this._id();
    const instData = instancesData[id];
    changeClientState(id, CONNECTING);
    const socket = module.connect(options);
    const connectionListener = createConnectionListener(this, type, id, options, socket);
    socket.on('connect', (socket) => {
      changeClientState(id, CONNECTED);
      connectionListener(socket);
    });
    socket.on('close', () => {
      changeClientState(id, CLOSED);
      this.free();
    });
    instData.data.internal = socket;
    return socket;
  }

  close() {
    const id = this._id();
    changeClientState(id, CLOSING);
    const instData = instancesData[id];
    const internal = instData.data.internal;
    internal.end();
  }
}

/**
 * @typedef {Object} BuilderOptions
 * @property {String} regexp - regular expression text
 * @property {String} grammar - grammar text
 * @property {Object} [proto] - prototype for message context
 */

/**
 * Creates and return an instance of Server or Client.
 * @param {BuilderOptions | Array<BuilderOptions>} options
 * @param {String} [type] - type of instance: 'server' or 'client', default is 'server'
 * @return {Server | Client}
 */
function build(options, type = 'server') {
  if (!Array.isArray(options)) options = [options];
  const execOptions = [];
  options.forEach((opt) => {
    const regexp = opt.regexp;
    assert(regexp, '"regexp" option must be set');
    const grammar = opt.grammar;
    assert(grammar, '"grammar" option must be set');
    const proto = opt.proto;
    const executor = Executor.build(regexp, grammar);
    execOptions.push({
      executor,
      proto
    });
  });

  let instance;
  if (type == 'server') {
    instance = new Server(execOptions);
  } else if (type == 'client') {
    instance = new Client(execOptions);
  } else {
    throw new Error(`Unknown type: ${type}`);
  }

  return instance;
}

module.exports = {
  build,
  hash
}
