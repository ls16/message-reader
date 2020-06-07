const grammar = require('./grammar');
const {build} = require('../../src');

function buildCalc() {
  return build({
    regexp: grammar.regexp,
    grammar: grammar.grammar,
    proto: grammar
  })
};

module.exports = {
  build: buildCalc
};
