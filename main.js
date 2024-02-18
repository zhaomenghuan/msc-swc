const { transformSync } = require('./index');

function toBuffer(t) {
  return Buffer.from(JSON.stringify(t));
}

const options = toBuffer({
  cwd: '',
  filename: 'index.js',
  sourceMaps: false,
  isModule: true,
  jsc: {
    parser: {
      syntax: 'ecmascript',
      jsx: false,
    },
    transform: {},
  },
  module: {
    type: 'commonjs',
    strictMode: true,
  },
});

const customOptions = toBuffer({
  externalPackages: ['react'],
});

const content = `
import fs from 'fs';
import { resolve } from 'path';
import { debounce } from 'lodash';
const a = require('../abc');
require('/def');
require('./../def');
export * from 'exp1';
export { ee } from 'exp1';
export const main = function(require1) {
    let test = require1("halou");
    return test;
};

`;
const result = transformSync(content, options, customOptions);
console.log(result);
