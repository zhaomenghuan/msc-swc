import path from 'path';
import fs from 'fs';
import test from 'ava';
import glob from 'fast-glob';
import { swcTransformSync } from '../index';

export function resolveFileType(fileName: string) {
  return path.extname(fileName).substring(1);
}

function toBuffer(t: unknown): Buffer {
  return Buffer.from(JSON.stringify(t));
}

function compile(cwd: string, filename: string, content: string) {
  const fileType = resolveFileType(filename);
  const enableTypescript = fileType === 'ts' || fileType === 'tsx';
  const enableJSX = fileType === 'jsx' || fileType === 'tsx';
  const opts = toBuffer({
    cwd,
    filename,
    sourceMaps: false,
    isModule: true,
    jsc: {
      parser: {
        syntax: enableTypescript ? 'typescript' : 'ecmascript',
        jsx: enableJSX,
      },
      transform: {},
    },
    module: {
      type: 'commonjs',
      strictMode: true,
    },
  });
  console.info('-----------------------------');
  const result = swcTransformSync(content, opts);
  console.info('filename: ', filename);
  console.info('code: \n', result.code);
  console.info('metadata:', result.metadata);
  console.info('-----------------------------');
  return result;
}

test('swcTransformSync 简单示例', (t) => {
  const content = `
import '../utils/index';
import './index.css';

function joinPath(name) {
  return path.join(__dirname, name);
}

const appPath = joinPath('__app');
console.log('appPath: ', appPath);`;
  const result = compile('', 'pages/index/index.js', content);
  t.is(result.metadata.requires.includes('/pages/utils/index.js'), true);
  t.pass();
});

test('swcTransformSync 完整示例', (t) => {
  const TEST_PROJECT_ROOT_PATH = path.join(__dirname, 'app');
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

test('swcTransformSync 错误提示示例', (t) => {
  const content = `
import '../utils/index';

function joinPath(name) {
  return path.join(__dirname, name);
}

const appPath = joinPath('__app');
console.log('appPath: ', appPath);`;
  try {
    compile(path.join(__dirname), 'pages/index/index.js', content);
  } catch (error) {
    console.error(error);
    t.pass();
  }
});
