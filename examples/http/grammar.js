let regexp = `
  ctl1 \\x00|[\\x01-\\x08]
  ht \\x09
  lf \\x0A
  ctl2 [\\x0B-\\x0C]
  cr \\x0D
  ctl3 [\\x0E-\\x1F]
  sp \\x20
  '!' \\x21
  '"' \\x22
  '#' \\x23
  '$' \\x24
  '%' \\x25
  '&' \\x26
  '\'' \\x27
  '(' \\x28
  ')' \\x29
  '*' \\x2A
  '+' \\x2B
  ',' \\x2C
  '-' \\x2D
  '.' \\x2E
  '/' \\x2F
  '0' \\x30
  '1' \\x31
  '2' \\x32
  '3' \\x33
  '4' \\x34
  '5' \\x35
  '6' \\x36
  '7' \\x37
  '8' \\x38
  '9' \\x39
  ':' \\x3A
  ';' \\x3B
  '<' \\x3C
  '=' \\x3D
  '>' \\x3E
  '?' \\x3F
  '@' \\x40
  A_F [\\x41-\\x46]
  G \\x47
  H \\x48
  I_O [\\x49-\\x4F]
  P \\x50
  Q_S [\\x51-\\x53]
  T \\x54
  U_Z [\\x55-\\x5A]
  '[' \\x5B
  '\\' \\x5C
  ']' \\x5D
  '^' \\x5E
  '_' \\x5F
  ga \\x60
  a_f [\\x61-\\x66]
  g \\x67
  h \\x68
  i_o [\\x69-\\x6F]
  p \\x70
  q_r [\\x71-\\x72]
  s \\x73
  t \\x74
  u \\x75
  v \\x76
  w_z [\\x77-\\x7A]
  '{' \\x7B
  '|' \\x7C
  '}' \\x7D
  '~' \\x7E
  ctl4 \\x7F
  obs_text [\\x80-\\xFF]
`;

let grammar = `
  start: http_message;

  http_message: start_line_headers crlf message_body;
  http_message: start_line_headers crlf;
  start_line_headers: start_line headers {this.readBody(push_after)};
  message_body: 'message_body_1' | chunked_body;
  headers: headers header_field_crlf;
  headers: ;

  http_name: 'H' 'T' 'T' 'P';

  http_version: http_name '/' 'digit' '.' 'digit' {this.readHttpVersion(get(2), get(0))};

  absolute_form: absolute_uri [set(0)];
  absolute_path: absolute_path_1 absolute_path [set(1, 0)];
  absolute_path: absolute_path_1 [set(0)];
  absolute_path_1: '/' segment [set(1, 0)];
  asterisk_form: '*' [set(0)];
  authority_form: authority [set(0)];

  chunk: chunk_size chunk_ext crlf chunk_data crlf;
  chunk: chunk_size crlf chunk_data crlf;
  chunk_data: 'chunk_data_1';
  chunk_ext: chunk_ext chunk_ext_1;
  chunk_ext: ;
  chunk_ext_1: ';' chunk_ext_name '=' chunk_ext_val;
  chunk_ext_1: ';' chunk_ext_name;
  chunk_ext_name: token;
  chunk_ext_val: token | quoted_string;
  chunk_size: chunk_size_1 {this.readChunk(get(0), push_after)};
  chunk_size_1: chunk_size_1_1 {set(0); if (this.isLastChunkSize(get(0))) set_name('last_chunk_size')};
  chunk_size_1_1: hex chunk_size_1_1 [set(1, 0)];
  chunk_size_1_1: hex [set(0)];
  chunked_body: chunked_body_1 crlf;
  chunked_body_1: chunked_body_1_1 last_chunk trailer_part {push_after('lf')};
  chunked_body_1_1: chunked_body_1_1 chunk;
  chunked_body_1_1: ;

  comment: '(' comment_1 ')' [set(2, 1, 0)];
  comment_1: comment_1 comment_1_1 [set(1, 0)];
  comment_1: ;
  comment_1_1: ctext | quoted_pair | comment [set(0)];
  ctext: 'ht' | 'sp' | char21_27 | char2A_5B | char5D_7E | obs_text [set(0)];

  field_name: token [set(0)];
  field_value: field_value field_value_1 [set(1, 0)];
  field_value: ;
  field_value_1: field_vchar | obs_fold | 'sp' | 'ht' [set(0)];
  field_vchar: vchar | obs_text [set(0)];

  header_field_crlf: field_name ':' field_value crlf {this.request.headers[Buffer.from(get(3)).toString().toLowerCase()] = Buffer.from(get(1)).toString().trim()};

  last_chunk: last_chunk_size chunk_ext crlf;
  last_chunk: last_chunk_size crlf;

  message_body: 'message_body_1';
  method: token {this.request.method = Buffer.from(get(0)).toString()};

  obs_fold: crlf obs_fold_1 [set(0)];
  obs_fold_1: obs_fold_1_1 obs_fold_1 [set(1, 0)];
  obs_fold_1: obs_fold_1_1 [set(0)];
  obs_fold_1_1: 'sp' | 'ht' [set(0)];
  origin_form: absolute_path '?' query [set(2, 1, 0)];
  origin_form: absolute_path [set(0)];

  partial_uri: relative_part '?' query [set(2, 1, 0)];
  partial_uri: relative_part [set(0)];
  protocol: protocol_name '/' protocol_version [set(2, 1, 0)];
  protocol: protocol_name [set(0)];
  protocol_name: token [set(0)];
  protocol_version: token [set(0)];
  pseudonym: token [set(0)];

  qdtext: 'ht' | 'sp' | '!' | char23_5B | char5D_7E | obs_text [set(0)];
  quoted_pair:  '\\' quoted_pair_1 [set(1, 0)];
  quoted_pair_1: 'ht' | 'sp' | vchar | obs_text [set(0)];
  quoted_string: '"' quoted_string_1 '"' [set(2, 1, 0)];
  quoted_string_1: quoted_string_1 quoted_string_1_1 [set(1, 0)];
  quoted_string_1: ;
  quoted_string_1_1: qdtext | quoted_pair [set(0)];

  request_line: method 'sp' request_target 'sp' http_version crlf;
  request_target: origin_form | absolute_form | authority_form | asterisk_form {this.request.url = Buffer.from(get(0)).toString()};

  start_line: request_line;

  tchar: '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '.' | '^' | '_' | 'ga' | '|' | '~' | digit | alpha [set(0)];
  token: token tchar [set(1, 0)];
  token: tchar [set(0)];
  trailer_part: trailer_part trailer_part_1;
  trailer_part: ;
  trailer_part_1: header_field_crlf;

  digit: '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' [set(0)];
  hex: 'A_F' | 'a_f' | digit [set(0)];
  alpha: 'A_F' | 'G' | 'H' | 'I_O' | 'P' | 'Q_S' | 'T' | 'U_Z' | 'a_f' | 'g' | 'h' | 'i_o' | 'p' | 'q_r' | 's' | 't' | 'u' | 'v' | 'w_z' [set(0)];
  char21_27: '!' | '"' | '#' | '$' | '%' | '&' | '\'' [set(0)];
  char23_5B: '#' | '$' | '%' | '&' | '\'' | '(' | ')' | char2A_5B [set(0)];
  char2A_5B: '*' | '+' | ',' | '-' | '.' | '/' | digit | ':' | ';' | '<' | '=' | '>' | '?' | '@' | 'A_F' | 'G' | 'H' | 'I_O' | 'P' | 'Q_S' | 'T' | 'U_Z' | '[' [set(0)];
  char5D_7E: ']' | '^' | '_' | 'ga' | 'a_f' | 'g' | 'h' | 'i_o' | 'p' | 'q_r' | 's' | 't' | 'u' | 'v' | 'w_z' | '{' | '|' | '}' | '~' [set(0)];
  vchar: char21_27 | char23_5B | '\\' | char5D_7E [set(0)];
  crlf: 'cr' 'lf';


  absolute_uri: scheme ':' hier_part '?' query [set(4, 3, 2, 1, 0)];
  absolute_uri: scheme ':' hier_part [set(2, 1, 0)];
  relative_part: '/' '/' authority path_abempty [set(3, 2, 1, 0)];
  relative_part: path_absolute [set(0)];
  relative_part: path_noscheme [set(0)];
  relative_part: path_empty [set(0)];
  scheme: alpha scheme_1 [set(1, 0)];
  scheme_1: scheme_1 scheme_1_1 [set(1, 0)];
  scheme_1: ;
  scheme_char: alpha | digit | '+' | '-' | '.' [set(0)];
  hier_part: '/' '/' authority path_abempty [set(3, 2, 1, 0)];
  hier_part: path_absolute | path_rootless | path_empty [set(0)];
  authority: userinfo '@' host ':' port [set(4, 3, 2, 1, 0)];
  authority: host [set(4, 3, 2, 1, 0)];
  userinfo: userinfo userinfo_1 [set(1, 0)];
  userinfo: ;
  userinfo_1: unreserved | pct_encoded | sub_delims | ':' [set(0)];
  host: ip_literal | ip_v4address | reg_name [set(0)];
  port: port digit [set(1, 0)];
  port: ;
  ip_literal: '[' ip_literal_1 ']' [set(2, 1, 0)];
  ip_literal_1: ip_v6address | ip_vfuture [set(0)];
  ip_vfuture: 'v' ip_vfuture_1 '.' ip_vfuture_2 [set(3, 2, 1, 0)];
  ip_vfuture_1: hex ip_vfuture_1 [set(1, 0)];
  ip_vfuture_1: hex [set(0)];
  ip_vfuture_2: ip_vfuture_2_1 ip_vfuture_2 [set(1, 0)];
  ip_vfuture_2_1: unreserved | sub_delims | ':' [set(0)];
  ip_v6address: ip_v6address_1 | ip_v6address_2 | ip_v6address_3 | ip_v6address_4 | ip_v6address_5 | ip_v6address_6 | ip_v6address_7 | ip_v6address_8 | ip_v6address_9 [set(0)];
  ip_v6address_1: h16_ext h16_ext h16_ext h16_ext h16_ext h16_ext ls32 {set_val(Buffer.concat([get(6), get(5), get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_2: ':' ':' h16_ext h16_ext h16_ext h16_ext h16_ext ls32 {set_val(Buffer.concat([get(7), get(6), get(5), get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_3: ip_v6address_3_1 ':' ':' h16_ext h16_ext h16_ext h16_ext ls32 {set_val(Buffer.concat([get(7), get(6), get(5), get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_3_1: h16 [set(0)];
  ip_v6address_3_1: ;
  ip_v6address_4: ip_v6address_4_1 ':' ':' h16_ext h16_ext h16_ext ls32 {set_val(Buffer.concat([get(6), get(5), get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_4_1: ip_v6address_4_1_1 h16 [set(1, 0)];
  ip_v6address_4_1: ;
  ip_v6address_4_1_1: h16_ext [set(0)];
  ip_v6address_4_1_1: ;
  ip_v6address_5: ip_v6address_5_1 ':' ':' h16_ext h16_ext ls32 {set_val(Buffer.concat([get(5), get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_5_1: ip_v6address_5_1_1 h16 [set(1, 0)];
  ip_v6address_5_1: ;
  ip_v6address_5_1_1: h16_ext h16_ext [set(1, 0)];
  ip_v6address_5_1_1: h16_ext [set(0)];
  ip_v6address_5_1_1: ;
  ip_v6address_6: ip_v6address_6_1 ':' ':' h16_ext ls32 [set(4, 3, 2, 1, 0)];
  ip_v6address_6_1: ip_v6address_6_1_1 h16 [set(1, 0)];
  ip_v6address_6_1: ;
  ip_v6address_6_1_1: h16_ext h16_ext h16_ext [set(2, 1, 0)];
  ip_v6address_6_1_1: h16_ext h16_ext [set(1, 0)];
  ip_v6address_6_1_1: h16_ext [set(0)];
  ip_v6address_6_1_1: ;
  ip_v6address_7: ip_v6address_7_1 ':' ':' ls32 [set(3, 2, 1, 0)];
  ip_v6address_7_1: ip_v6address_7_1_1 h16 [set(1, 0)];
  ip_v6address_7_1: ;
  ip_v6address_7_1_1: h16_ext h16_ext h16_ext h16_ext [set(3, 2, 1, 0)];
  ip_v6address_7_1_1: h16_ext h16_ext h16_ext [set(2, 1, 0)];
  ip_v6address_7_1_1: h16_ext h16_ext [set(1, 0)];
  ip_v6address_7_1_1: h16_ext [set(0)];
  ip_v6address_7_1_1: ;
  ip_v6address_8: ip_v6address_8_1 ':' ':' h16 [set(3, 2, 1, 0)];
  ip_v6address_8_1: ip_v6address_8_1_1 h16 [set(1, 0)];
  ip_v6address_8_1: ;
  ip_v6address_8_1_1: h16_ext h16_ext h16_ext h16_ext h16_ext [set(4, 3, 2, 1, 0)];
  ip_v6address_8_1_1: h16_ext h16_ext h16_ext h16_ext [set(3, 2, 1, 0)];
  ip_v6address_8_1_1: h16_ext h16_ext h16_ext [set(2, 1, 0)];
  ip_v6address_8_1_1: h16_ext h16_ext [set(1, 0)];
  ip_v6address_8_1_1: h16_ext [set(0)];
  ip_v6address_8_1_1: ;
  ip_v6address_9: ip_v6address_9_1 ':' ':' [set(2, 1, 0)];
  ip_v6address_9_1: ip_v6address_9_1_1 h16 [set(1, 0)];
  ip_v6address_9_1: ;
  ip_v6address_9_1_1: h16_ext h16_ext h16_ext h16_ext h16_ext h16_ext {set_val(Buffer.concat([get(5), get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_9_1_1: h16_ext h16_ext h16_ext h16_ext h16_ext [set(4, 3, 2, 1, 0)];
  ip_v6address_9_1_1: h16_ext h16_ext h16_ext h16_ext [set(3, 2, 1, 0)];
  ip_v6address_9_1_1: h16_ext h16_ext h16_ext [set(2, 1, 0)];
  ip_v6address_9_1_1: h16_ext h16_ext [set(1, 0)];
  ip_v6address_9_1_1: h16_ext [set(0)];
  ip_v6address_9_1_1: ;
  h16_ext: h16 ':' [set(1, 0)];
  h16: hex hex hex hex [set(3, 2, 1, 0)];
  h16: hex hex hex [set(2, 1, 0)];
  h16: hex hex [set(1, 0)];
  h16: hex [set(0)];
  ls32: ls32_1 | ip_v4address [set(0)];
  ls32_1: h16_ext h16 [set(1, 0)];
  ip_v4address: dec_octet_ext dec_octet_ext dec_octet_ext dec_octet [set(3, 2, 1, 0)];
  dec_octet_ext: dec_octet ':' [set(1, 0)];
  dec_octet: '1' digit digit [set(2, 10)];
  dec_octet: '2' dec_octet_1 digit [set(2, 1, 0)];
  dec_octet_1: '0' | '1' | '2' | '3' | '4' [set(0)];
  dec_octet: '2' '5' dec_octet_2 digit [set(3, 2, 1, 0)];
  dec_octet_2: '0' | '1' | '2' | '3' | '4' | '5' [set(0)];
  reg_name: reg_name reg_name_1 [set(1, 0)];
  reg_name: ;
  reg_name_1: unreserved | pct_encoded | sub_delims [set(0)];
  path_abempty: segments [set(0)];
  path_absolute: '/' path_absolute_1 [set(1, 0)];
  path_absolute_1: segment_nz segments [set(1, 0)];
  path_absolute_1: ;
  path_noscheme: segment_nz_nc segments [set(1, 0)];
  path_rootless: segment_nz segments [set(1, 0)];
  path_empty: ;
  segments: segments segments_1 [set(1, 0)];
  segments: ;
  segments_1: '/' segment [set(1, 0)];
  segment: segment pchar [set(1, 0)];
  segment: ;
  segment_nz: pchar segment_nz [set(1, 0)];
  segment_nz: pchar [set(0)];
  segment_nz_nc: segment_nz_nc_1 segment_nz_nc [set(1, 0)];
  segment_nz_nc: segment_nz_nc_1 [set(0)];
  segment_nz_nc_1: unreserved | pct_encoded | sub_delims | '@' [set(0)];
  pchar: unreserved | pct_encoded | sub_delims | ':' | '@' [set(0)];
  query: query_1 {set(0); let val = get(0); if (val.length > 0) this.request.query = Buffer.from(val).toString()};
  query_1: query_1 query_1_1 [set(1, 0)];
  query_1: ;
  query_1_1: pchar | '/' | '?' [set(0)];
  fragment: fragment fragment_1 [set(1, 0)];
  fragment: ;
  fragment_1: pchar | '/' | '?' [set(0)];
  pct_encoded: '%' hex hex [set(2, 1, 0)];
  unreserved: alpha | digit | '-' | '.' | '_' | '~' [set(0)];
  reserved: gen_delims | sub_delims [set(0)];
  gen_delims: ':' | '/' | '?' | '#' | '[' | ']' | '@';
  sub_delims: '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' [set(0)];
`;

function initRequest() {
  this.request = {};
  this.request.headers = {};
}

function doTknData(tknName, tknData, end) {
  //console.log(`tknData len: `, tknData.length);
  /*
  console.log(`tknData ----------`)
  console.log(Buffer.from(tknData).toString());
  if (end == true) {
    console.log('end of tknData ---');
  }
  */
}

function readHttpVersion(majorBytes, minorBytes) {
  this.request.httpVersion = `${Buffer.from(majorBytes).toString()}.${Buffer.from(minorBytes).toString()}`;
}

function readBody(push_after) {
  const contentLength = this.request.headers['content-length'];
  const transferEncoding = this.request.headers['transfer-encoding'];
  if (contentLength != null && transferEncoding == 'chunked') {
    throw new Error(`Headers 'content-length' and 'transfer-encoding (chunked)' are set.`);
  }

  if (transferEncoding == 'chunked') {
  } else {
    if (contentLength != null && +contentLength > 0) {
      const size = +contentLength;
      push_after('lf', 'message_body_1', null, size);
      push_after('message_body_1');
    } else {
      push_after('lf');
    }
  }
}

function readChunk(data, push_after) {
  const size = parseInt(Buffer.from(data).toString(), 16);
  push_after('lf', 'chunk_data_1', null, size);
}

function isLastChunkSize(data) {
  for (let i = 0; i < data.length; i++) {
    if (data[0] != 48) return false;
  }
  return true;
}

module.exports = {
  regexp,
  grammar,
  onBeforeParse: initRequest,
  onTknData: doTknData,
  readHttpVersion,
  readBody,
  readChunk,
  isLastChunkSize
};
