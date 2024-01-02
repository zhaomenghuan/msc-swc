import path from 'path';
import test from 'ava';
import { minifySync } from '../index';
import { compile, toBuffer } from '../test-utils';

test('transformSync Node.js 内置模块及外部依赖的示例', (t) => {
  const content = `
import path from 'path';
import { copyFile } from 'fs/promises';
import { writeFile } from 'node:fs/promises';
import React from 'react';

const newPath = path.join(__dirname, 'pages');
console.log(newPath);
  `;
  const { code } = compile('', 'pages/index/index.js', content);
  const actual =
    code.includes('require("path")') &&
    code.includes('require("fs/promises")') &&
    code.includes('require("node:fs/promises")') &&
    code.includes('require("react")');
  t.assert(actual);
});

test('transformSync 简单示例', (t) => {
  const content = `
import { log } from '../utils/index';
import './index.css';

log('hello, swc');`;
  compile('', 'pages/index/index.js', content);
  t.pass();
});

test('transformSync 错误提示示例', (t) => {
  const content = `
  import { log } from '../utils/index';

  log('hello, swc');
  `;
  try {
    compile(path.join(__dirname), 'pages/index/index.js', content);
  } catch (error) {
    console.error(error);
    t.pass();
  }
});

test('transformSync require 模块并立即调用模块方法', (t) => {
  const content = `
    require('./utils/util').formatTime(new Date());
  `;

  compile('', 'pages/index/index.js', content);
  t.pass();
});

test('minifySync 示例', (t) => {
  const code = `
    function deadCode() {
      console.log('dead code');
    }

    function log(message) {
      console.info('message: ' + message);
    }

    log('hello, swc');
  `;
  const options = {
    mangle: {
      safari10: true,
    },
    compress: {
      pure_funcs: ['console.log'],
    },
  };
  const result = minifySync(toBuffer(code), toBuffer(options));
  const expected = 'function deadCode(){}function log(o){console.info("message: "+o)}log("hello, swc");';
  t.is(result.code, expected);
});
