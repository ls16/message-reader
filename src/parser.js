exports.do_parser_action = (func, context, bind, id, get, set, set_val, set_name, push_after) => {
  func.call(context, bind, id, get, set, set_val, set_name, push_after);
}