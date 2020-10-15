exports.do_parser_action = (func, context, bind, id, lookup, get, set, set_val, set_name, set_name_from_hash, push_after) => {
  func.call(context, bind, id, lookup, get, set, set_val, set_name, set_name_from_hash, push_after);
}