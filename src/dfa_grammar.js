const data = {};

exports.cast_exec_context_to_js_value = (id, context) => {
  data[id] = context;
  return data[id];
}

exports.return_exec_context = (id) => {
  const context = data[id];
  delete data[id];
  return context;
}