const assert = require('assert');
const Net = require('net');
const Tls = require('tls');
const EventEmitter = require('events');
const {Executor, hash} = require('../pkg/server');

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

function createConnectionListener(self, type, id, options, sock) {
  const execMiddlewares = compose(id);
  const instData = instancesData[id];
  const timerIds = new Map();
  const parseTimeout = options.parseTimeout != null ? options.parseTimeout * 1000 : null;

  function timeOutHandler(ctx) {
    ctx.socket && ctx.socket.emit('__parse_init__');
    self.emit('error', {
      error: 'Timeout',
      ctx
    });
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
    };
  });

  return (socket) => {
    let optionsIndex = 0;
    let executor;

    function setOptions(index) {
      optionsIndex = index;
      executor = type == 'server'
        ? instData.execOptions[optionsIndex].master.clone_executor()
        : instData.execOptions[optionsIndex].master;
    }

    setOptions(0);
    if (type == 'client') socket = sock;

    socket.on('data', (data) => {
      try {
        executor.parse_data(data, instData.execOptions[optionsIndex].proto, 'socket', socket,
        instData.execOptions[optionsIndex].onBeforeParse,
        instData.execOptions[optionsIndex].onAfterParse,
        instData.execOptions[optionsIndex].proto.onTknData);
      } catch (err) {
        executor.parse_init();
        self.emit('error', {
          error: err,
          socket
        });
      }
    });
  
    socket.on('__parse_init__', () => {
      executor.parse_init();
    });

    socket.on('__set_options_index__', (index) => {
      if (index >= 0 && index < instData.execOptions.length) {
        setOptions(index);
      }
    });

    socket.on('error', (evt) => {
    });
  
    socket.on('close', () => {
      executor.free();
      executor = null;
    });
  };
}

class Base extends EventEmitter {
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
    super();

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
      middlewares: []
    };
  }

  free() {
    const id = this._id();
    const execOptions = instancesData[id].execOptions;
    execOptions.forEach((opt) => {
      opt.master && opt.master.free();
    });
    instancesData[id].execOptions = [];
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
    const instData = instancesData[this._id()];
    instData.middlewares.push(middleware);
    return this;
  }
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
    const connectionListener = createConnectionListener(this, type, id, options);
    const internal = module.createServer(createOptions, connectionListener);

    internal.maxConnections = 100;
    internal.listen(options);
  }
}

class Client extends Base {
  connect(options) {
    const module = options.tls === true ? Tls : Net;
    const type = 'client';
    const id = this._id();
    const socket = module.connect(options);
    const connectionListener = createConnectionListener(this, type, id, options, socket);
    socket.on('connect', connectionListener);
    return socket;
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
