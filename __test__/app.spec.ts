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
