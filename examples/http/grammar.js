const {hash} = require('../../src');

const hexdig = hash('hexdig');
const alpha_1 = hash('alpha_1');
const digit_1 = hash('digit_1');

const regexp = `
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
  '0' \\x30 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '1' \\x31 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '2' \\x32 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '3' \\x33 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '4' \\x34 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '5' \\x35 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '6' \\x36 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '7' \\x37 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '8' \\x38 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '9' \\x39 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  ':' \\x3A
  ';' \\x3B
  '<' \\x3C
  '=' \\x3D
  '>' \\x3E
  '?' \\x3F
  '@' \\x40
  A_F [\\x41-\\x46] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  G \\x47 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  H \\x48 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  I_O [\\x49-\\x4F] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  P \\x50 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  Q_S [\\x51-\\x53] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  T \\x54 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  U_Z [\\x55-\\x5A] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '[' \\x5B
  '\\' \\x5C
  ']' \\x5D
  '^' \\x5E
  '_' \\x5F
  ga \\x60
  a_f [\\x61-\\x66] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  g \\x67 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  h \\x68 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  i_o [\\x69-\\x6F] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  p \\x70 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  q_r [\\x71-\\x72] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  s \\x73 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  t \\x74 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  u \\x75 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  v \\x76 {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  w_z [\\x77-\\x7A] {(this.ipLiteralReading || this.requestTargetReading) && this.correctToken(get, set_name_from_hash)}
  '{' \\x7B
  '|' \\x7C
  '}' \\x7D
  '~' \\x7E
  ctl4 \\x7F
  obs_text [\\x80-\\xFF]
`;

const grammar = (isRequest) => `
  start: http_message;

  http_message: start_line_headers crlf message_body;
  http_message: start_line_headers crlf;
  start_line_headers: start_line headers {this.readBody(push_after)};
  message_body: 'message_body_1' | chunked_body;
  headers: headers header_field_crlf;
  headers: ;

  http_name: 'H' 'T' 'T' 'P';

  http_version: http_name '/' digit '.' digit {this.readHttpVersion(get(2), get(0))};

  absolute_form: absolute_uri [set(0)];
  absolute_path: absolute_path_1 absolute_path [set(1, 0)];
  absolute_path: absolute_path_1 [set(0)];
  absolute_path_1: '/' segment [set(1, 0)];
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
  ctext: 'ht' | 'sp' | char21_27 | char2A_5B | char5D_7E | 'obs_text' [set(0)];

  field_name: token [set(0)];
  field_value: field_value field_value_1 [set(1, 0)];
  field_value: ;
  field_value_1: field_vchar | 'sp' | 'ht' [set(0)];
  field_vchar: vchar | 'obs_text' [set(0)];

  header_field_crlf: field_name ':' field_value crlf {this.request.headers[Buffer.from(get(3)).toString().toLowerCase()] = Buffer.from(get(1)).toString().trim()};

  last_chunk: last_chunk_size chunk_ext crlf;
  last_chunk: last_chunk_size crlf;

  message_body: 'message_body_1';
  method: token {this.request.method = Buffer.from(get(0)).toString()};

  origin_form: absolute_path '?' query [set(2, 1, 0)];
  origin_form: absolute_path [set(0)];

  partial_uri: relative_part '?' query [set(2, 1, 0)];
  partial_uri: relative_part [set(0)];
  protocol: protocol_name '/' protocol_version [set(2, 1, 0)];
  protocol: protocol_name [set(0)];
  protocol_name: token [set(0)];
  protocol_version: token [set(0)];
  pseudonym: token [set(0)];

  qdtext: 'ht' | 'sp' | '!' | char23_5B | char5D_7E | 'obs_text' [set(0)];
  quoted_pair:  '\\' quoted_pair_1 [set(1, 0)];
  quoted_pair_1: 'ht' | 'sp' | vchar | 'obs_text' [set(0)];
  quoted_string: '"' quoted_string_1 '"' [set(2, 1, 0)];
  quoted_string_1: quoted_string_1 quoted_string_1_1 [set(1, 0)];
  quoted_string_1: ;
  quoted_string_1_1: qdtext | quoted_pair [set(0)];

  request_line: request_line_1 request_target 'sp' http_version crlf;
  request_line_1: method 'sp' {this.requestTargetReading = true};
  request_target: origin_form | absolute_form | authority_form | asterisk_form {this.requestTargetReading = false; this.request.url = Buffer.from(get(0)).toString()};

  start_line: ${isRequest == true ? 'request_line' : 'status_line'};
  status_line: http_version 'sp' status_code 'sp' reason_phrase crlf;
  status_code: digit digit digit {this.status = +(Buffer.concat([get(2), get(1), get(0)]).toString())};
  reason_phrase: reason_phrase_1 {this.reason = Buffer.from(get(0)).toString()};
  reason_phrase_1: reason_phrase_1 reason_phrase_1_1 [set(1, 0)];
  reason_phrase_1: ;
  reason_phrase_1_1: 'ht' | 'sp' | vchar | 'obs_text' [set(0)];

  tchar: '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '.' | '^' | '_' | 'ga' | '|' | '~' | digit | alpha [set(0)];
  token: token tchar [set(1, 0)];
  token: tchar [set(0)];
  trailer_part: trailer_part trailer_part_1;
  trailer_part: ;
  trailer_part_1: header_field_crlf;

  hex: 'A_F' | 'a_f' | digit [set(0)];
  digit: '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' [set(0)];
  alpha: 'A_F' | 'G' | 'H' | 'I_O' | 'P' | 'Q_S' | 'T' | 'U_Z' | 'a_f' | 'g' | 'h' | 'i_o' | 'p' | 'q_r' | 's' | 't' | 'u' | 'v' | 'w_z' [set(0)];
  char21_27: '!' | '"' | '#' | '$' | '%' | '&' | '\'' [set(0)];
  char23_5B: '#' | '$' | '%' | '&' | '\'' | '(' | ')' | char2A_5B [set(0)];
  char2A_5B: '*' | '+' | ',' | '-' | '.' | '/' | digit | ':' | ';' | '<' | '=' | '>' | '?' | '@' | 'A_F' | 'G' | 'H' | 'I_O' | 'P' | 'Q_S' | 'T' | 'U_Z' | '[' [set(0)];
  char5D_7E: ']' | '^' | '_' | 'ga' | 'a_f' | 'g' | 'h' | 'i_o' | 'p' | 'q_r' | 's' | 't' | 'u' | 'v' | 'w_z' | '{' | '|' | '}' | '~' [set(0)];
  vchar: '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '#' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | digit | ':' | ';' | '<' | '=' | '>' | '?' | '@' | 'A_F' | 'G' | 'H' | 'I_O' | 'P' | 'Q_S' | 'T' | 'U_Z' | '[' | '\\' | ']' | '^' | '_' | 'ga' | 'a_f' | 'g' | 'h' | 'i_o' | 'p' | 'q_r' | 's' | 't' | 'u' | 'v' | 'w_z' | '{' | '|' | '}' | '~' [set(0)];
  crlf: 'cr' 'lf';


  absolute_uri: scheme ':' hier_part '?' query [set(4, 3, 2, 1, 0)];
  absolute_uri: scheme ':' hier_part [set(2, 1, 0)];
  relative_part: '/' '/' authority path_abempty [set(3, 2, 1, 0)];
  relative_part: path_absolute [set(0)];
  relative_part: path_noscheme [set(0)];
  relative_part: path_empty [set(0)];
  scheme: alpha scheme_1 [set(1, 0)];
  scheme_1: scheme_1 scheme_char [set(1, 0)];
  scheme_1: ;
  scheme_char: 'alpha_1' | 'digit_1' | '+' | '-' | '.' [set(0)];
  hier_part: '/' '/' authority path_abempty [set(3, 2, 1, 0)];
  hier_part: path_absolute | path_rootless | path_empty [set(0)];
  authority: host authority_1 [set(1, 0)];
  authority_1: ':' port [set(1, 0)];
  authority_1: ;
  userinfo_1: unreserved | pct_encoded | sub_delims | ':' [set(0)];
  host: ip_literal | ip_v4address | reg_name [set(0)];
  port: port 'digit_1' [set(1, 0)];
  port: ;
  ip_literal: ip_literal_1 '[' ip_literal_2 ']' [set(2, 1, 0)];
  ip_literal_1: {this.ipLiteralReading = true};
  ip_literal_2: ip_v6address | ip_vfuture {this.ipLiteralReading = false; set(0)};
  ip_vfuture: 'v' ip_vfuture_1 '.' ip_vfuture_2 [set(3, 2, 1, 0)];
  ip_vfuture_1: 'hexdig' ip_vfuture_1 [set(1, 0)];
  ip_vfuture_1: 'hexdig' [set(0)];
  ip_vfuture_2: ip_vfuture_2_1 ip_vfuture_2 [set(1, 0)];
  ip_vfuture_2_1: unreserved | sub_delims | ':' [set(0)];
  ip_v6address: ip_v6address_1 [set(0)];
  ip_v6address_1: ip_v6_part2 ls32  {if (this.loopCountPart2 != 6) throw new Error('Invalid IP V6 addr'); set_val(Buffer.concat([get(1), get(0)]))};
  ip_v6address_1: ':' ':' ip_v6_part2 ls32 {if (this.loopCountPart2 != 5) throw new Error('Invalid IP V6 addr'); set_val(Buffer.concat([get(3), get(2), get(1), get(0)]))};
  ip_v6address_1: ip_v6_part1 ':' ':' ip_v6_part2 ls32 {this.checkIPV6(); set_val(Buffer.concat([get(4), get(3), get(2), get(1), get(0)]))};
  ip_v6address_1: ':' ':' ls32 [set(2, 1, 0)];
  ip_v6address_1: ip_v6_part1 ':' ':' ls32 {if (this.loopCountPart1 > 5) throw new Error('Invalid IP V6 addr'); set_val(Buffer.concat([get(2), get(1), get(0)]))};
  ip_v6address_1: ':' ':' h16 [set(2, 1, 0)];
  ip_v6address_1: ip_v6_part1 ':' ':' h16 {if (this.loopCountPart1 > 5) throw new Error('Invalid IP V6 addr'); set_val(Buffer.concat([get(2), get(1), get(0)]))};
  ip_v6address_1: ':' ':' [set(1, 0)];
  ip_v6address_1: ip_v6_part1 ':' ':' {if (this.loopCountPart1 > 6) throw new Error('Invalid IP V6 addr'); set_val(Buffer.concat([get(2), get(1), get(0)]))};
  ip_v6_part2: h16_colon_loop2 {this.loopCountPart2 = this.loopCount; set(0)};
  ip_v6_part1: h16_colon_loop1 h16 {this.loopCountPart1 = this.loopCount; set_val(Buffer.concat([get(1), get(0)]))};
  h16_colon_loop1: h16_colon_loop1_1 {this.loopCountPart1 != null && set_name('h16_colon_loop2')};
  h16_colon_loop1_1: h16 ':' h16_colon_loop1_1 {this.loopCount += 1; set_val(Buffer.concat([get(2), get(1), get(0)]))};
  h16_colon_loop1_1:  {this.loopCount = 0};
  ls32: ls32_1 | ip_v4address [set(0)];
  ls32_1: h16 ls32_1_1 [set(1, 0)];
  ls32_1_1: ':' h16 [set(1, 0)];
  h16: 'hexdig' 'hexdig' 'hexdig' 'hexdig' [set(3, 2, 1, 0)];
  h16: 'hexdig' 'hexdig' 'hexdig' [set(2, 1, 0)];
  h16: 'hexdig' 'hexdig' [set(1, 0)];
  h16: 'hexdig' [set(0)];
  ip_v4address: dec_octet_ext dec_octet_ext dec_octet_ext dec_octet_ext [set(3, 2, 1, 0)];
  dec_octet_ext: dec_octet '.' [set(1, 0)];
  dec_octet: 'digit_1' {this.checkDecOctet(get, 1); set(0)};
  dec_octet: 'digit_1' 'digit_1' {this.checkDecOctet(get, 1); set_val(Buffer.concat([get(1), get(0)]))};
  dec_octet: 'digit_1' 'digit_1' 'digit_1' {this.checkDecOctet(get, 1); set_val(Buffer.concat([get(2), get(1), get(0)]))};

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
  segment_nz: segment_nz pchar [set(1, 0)];
  segment_nz: pchar [set(0)];
  segment_nz_nc: segment_nz_nc segment_nz_nc_1 [set(1, 0)];
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
  pct_encoded: '%' hex1 hex1 [set(2, 1, 0)];
  hex1: 'alpha_1' | 'digit_1' {this.checkHex(get(0)[0]); set(0)};
  unreserved: 'alpha_1' | 'digit_1' | '-' | '.' | '_' | '~' [set(0)];
  reserved: gen_delims | sub_delims [set(0)];
  gen_delims: ':' | '/' | '?' | '#' | '[' | ']' | '@' [set(0)];
  sub_delims: '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' [set(0)];
`;

const requestGrammar = grammar(true);

const responseGrammar = grammar(false);

function initRequest() {
  this.request = {};
  this.request.headers = {};

  this.loopCount = null;
  this.loopCountPart1 = null;
  this.loopCountPart2 = null;
  this.requestTargetReading = false;
  this.ipLiteralReading = false;
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

function correctToken(get, set_name_from_hash) {
  if (!this.ipLiteralReading && !this.requestTargetReading) return;

  const code = get()[0];
  if (code >= 0x30 && code <= 0x39) {
    this.ipLiteralReading ? set_name_from_hash(hexdig) : set_name_from_hash(digit_1);
  } else if ((code >= 0x41 && code <= 0x5A) || (code >= 0x61 && code <= 0x7A)) {
    if (this.ipLiteralReading) {
      if ((code >= 0x41 && code <= 0x46) || (code >= 0x61 && code <= 0x66)) {
        set_name_from_hash(hexdig);
      } else {
        set_name_from_hash(alpha_1);
      }
    } else {
      set_name_from_hash(alpha_1);
    }
  }
}

function checkIPV6() {
  if (this.loopCountPart1 == 0 && this.loopCountPart2 == 4) return;
  if (this.loopCountPart1 <= 1 && this.loopCountPart2 == 3) return;
  if (this.loopCountPart1 <= 2 && this.loopCountPart2 == 2) return;
  if (this.loopCountPart1 <= 3 && this.loopCountPart2 == 1) return;
  throw new Error('Invalid IP V6 addr');
}

function checkDecOctet(get, len) {
  let valid = false;
  let digit0;
  let digit1;
  let digit2;

  switch (len) {
    case 1:
      valid = get(0) <= 0x39;
      break;
    case 2:
      digit1 = get(1);
      digit0 = get(0);
      valid = 
        (digit1 >= 0x31 && digit1 <= 0x39) &&
        (digit0 >= 0x30 && digit0 <= 0x39);
      break;
    case 3:
      digit2 = get(2);
      digit1 = get(1);
      digit0 = get(0);
      if (digit2 == 0x31) {
        valid = 
        (digit1 >= 0x30 && digit1 <= 0x39) &&
        (digit0 >= 0x30 && digit0 <= 0x39);
      } else if (digit2 == 0x32) {
        if (digit1 >= 0x30 && digit1 <= 0x34) {
          valid = (digit0 >= 0x30 && digit0 <= 0x39);
        } else if (digit1 == 0x35) {
          valid = (digit0 >= 0x30 && digit0 <= 0x35);
        }
      }
      break;
    }
  if (!valid) throw new Error('Invalid decimal digit');
}

function checkHex(code) {
  const valid = (code >= 0x30 && code <= 0x39) || (code >= 0x41 && code <= 0x46) || (code >= 0x61 && code <= 0x66);
  if (!valid) throw new Error('Invalid hex digit');
}

module.exports = {
  regexp,
  requestGrammar,
  responseGrammar,
  onBeforeParse: initRequest,
  onTknData: doTknData,
  readHttpVersion,
  readBody,
  readChunk,
  isLastChunkSize,
  correctToken,
  checkIPV6,
  checkDecOctet,
  checkHex
};
