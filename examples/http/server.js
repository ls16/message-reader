const grammar = require('./grammar');
const {build} = require('../../src');

function buildHttp() {
  return build({
    regexp: grammar.regexp,
    grammar: grammar.requestGrammar,
    proto: grammar
  })
};

module.exports = {
  build: buildHttp
};
