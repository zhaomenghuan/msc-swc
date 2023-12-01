import path from 'path';
import fs from 'fs';
import test from 'ava';
import glob from 'fast-glob';
import { swcTransformSync, minifySync } from '../index';

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
import { log } from '../utils/index';
import './index.css';

log('hello, swc');`;
  compile('', 'pages/index/index.js', content);
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
