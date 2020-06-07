let regexp = `
lf          \\x0A
cr          \\x0D
sp          \\x20      DEF
space       {sp}+           {pass()}
letter      [A-Za-z]   DEF
digit       [0-9]      DEF
func        {letter}({letter}|{digit})+
number      {digit}+(\\.{digit}*)?((e|E)(\\+|\\-)?{digit}+)?
'+'         \\+
'-'         \\-
'*'         \\*
'/'         \\x2F
'('         \\(
')'         \\)
'^'         \\^
`;

let grammar = `
  start: E2;
  E2: E1 EOL                {this.result = get(1);};
  EOL: 'cr' 'lf' | 'lf'     ;
  E1: E                     {set(0); push_after('lf')};
  E: E '+' T                {set_val(this.op('+', get(2), get(0)))};
  E: E '-' T                {set_val(this.op('-', get(2), get(0)))};
  E: T                      {set(0)};
  T: T '*' F                {set_val(this.op('*', get(2), get(0)))};
  T: T '/' F                {set_val(this.op('/', get(2), get(0)))};
  T: T '^' F                {set_val(this.op('^', get(2), get(0)))};
  T: '-' F                  {set_val(this.unary(get(0)))};
  T: F                      {set(0)};
  F: '(' E ')'              {set(1)};
  F: 'number'               {set(0)};
  F: 'func' '(' E ')'       {set_val(this.func(get(3), get(1)))};
`;

function arrayToString(value) {
  return [...value].map(val => String.fromCharCode(val)).join('');
}

function numberToArray(value) {
  return value.toString().split('').map(val => val.charCodeAt());
}

function arrayToNumber(value) {
  return parseFloat(arrayToString(value));
}

function op(opName, a, b) {
  let a1 = arrayToNumber(a);
  let b1 = arrayToNumber(b);
  let res;
  switch (opName) {
    case '+':
      res = a1 + b1;
      break;
    case '-':
      res = a1 - b1;
      break;
    case '*':
      res = a1 * b1;
      break;
    case '/':
      res = a1 / b1;
      break;
    case '^':
      res = Math.pow(a1, b1);
      break;
    }
  return numberToArray(res);
}

function unary(a) {
  return numberToArray(-arrayToNumber(a));
}

function func(name, a) {
  name = arrayToString(name).toLowerCase();
  const namesArr = ['abs', 'sin', 'cos', 'tan', 'exp', 'log2'];
  if (namesArr.indexOf(name) == -1) throw new Error(`Unknown function '${name}'`);
  const a1 = arrayToNumber(a);
  return numberToArray(Math[name](a1));
}

module.exports = {
  regexp,
  grammar,
  op,
  unary,
  func
};