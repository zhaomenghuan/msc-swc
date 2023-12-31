import path from 'path';
import fs from 'fs';
import glob from 'fast-glob';
import test from 'ava';
import { compile } from '../test-utils';

const TEST_PROJECT_ROOT_PATH = path.join(__dirname, 'app');

test('transformSync 完整示例', (t) => {
  const files = glob.sync('**/*.+(js|jsx|ts|tsx)', {
    cwd: TEST_PROJECT_ROOT_PATH,
    onlyFiles: true,
  });

  files.forEach((filename) => {
    const content = fs.readFileSync(path.join(TEST_PROJECT_ROOT_PATH, filename), 'utf8');
    compile(TEST_PROJECT_ROOT_PATH, filename, content);
  });

  t.pass();
});

test('绝对路径引用', (t) => {
  const result = compile(TEST_PROJECT_ROOT_PATH, 'pages/log/log.ts');
  t.assert(result?.code?.includes(`require("/utils/util.js")`));
});

test('使用 ts 文件并且省略 index', (t) => {
  const result = compile(TEST_PROJECT_ROOT_PATH, 'analyze.ts');
  t.assert(result?.code?.includes(`require("/module/index.js")`));
});

test('使用 npm 包', (t) => {
  const result = compile(TEST_PROJECT_ROOT_PATH, 'utils/util.ts');
  t.assert(result?.code?.includes(`require("/node_modules/@msc/utils/index.js")`));
});

test('transformSync 编译引用不存在的 npm ', (t) => {
  const content = `
    const { debug } = require('@msc/msc-utils');

    export function log(message: string) {
      debug(message);
    }
  `;
  try {
    compile(TEST_PROJECT_ROOT_PATH, 'utils/util.ts', content);
  } catch (error) {
    console.error(error);
    t.pass();
  }
});
