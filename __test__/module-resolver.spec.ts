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

test('transformSync require 作为参数', (t) => {
  const content = `
    const mod_a = require('./utils/util.ts')
    function scopeRequire(require, b) {
        require('aaa')
    
        function scopeRequireB(require) {
        require('bbb')
        }
    
        scopeRequireB(req)
    }
    
    __DEFINE__(
        1640576341559,
        function (require, module, exports) {
        const Logan = require('@dp/logan-wxapp')
    
        function config(config) {
            Logan.config(config)
        }
    
        function write(log) {
            log = log || {}
            let logString = log.logString || ''
            let logType = log.logType || 'default'
            if (typeof logString !== 'string' || typeof logType !== 'string') {
            return
            }
            Logan.log(logString, logType)
        }
    
        function event(log) {
            console.log('log event not support.')
        }
    
        function upload(config) {
            Logan.report(config)
        }
    
        function flush() {
            console.log('flush not support.')
        }
    
        module.exports = {
            config,
            write,
            event,
            upload,
            flush,
        }
        },
        function (modId) {
        var map = {}
        return __REQUIRE__(map[modId], modId)
        }
    )
    
    const result = scopeRequire(req, param)
    const mod_b = require('./pages/index/index.js')  
  `;
  const result = compile(TEST_PROJECT_ROOT_PATH, 'app.js', content);
  const requires = result?.metadata?.requires;
  t.assert(Array.isArray(requires) && requires.length === 2);
});

test('transformSync 收集 export from 依赖', (t) => {
  const content = `
      export {test} from './utils/util.ts'
    `;
  const result = compile(TEST_PROJECT_ROOT_PATH, 'app.js', content);
  const requires = result?.metadata?.requires;
  t.assert(Array.isArray(requires) && requires.length === 1);
});
