const grammar = require('./grammar');
const {build} = require('../../src');

function buildHttp() {
  return build({
    regexp: grammar.regexp,
    grammar: grammar.grammar,
    proto: grammar
  })
};

module.exports = {
  build: buildHttp
};
