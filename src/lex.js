exports.do_lex_action = (func, context, get, set, set_name, pass) => {
  func.call(context, get, set, set_name, pass);
}