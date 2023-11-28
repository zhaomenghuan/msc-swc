import test from 'ava'

import { swcTransformSync } from '../index'

function toBuffer(t: any): Buffer {
  return Buffer.from(JSON.stringify(t))
}

function compile(filePath: string, content: string) {
  const opts = toBuffer({
    filename: filePath,
    sourceMaps: false,
    isModule: true,
    jsc: {
      parser: {
        syntax: 'ecmascript',
        jsx: false,
      },
      transform: {},
      experimental: {
        plugins: [],
      },
    },
    module: {
      type: 'commonjs',
      strictMode: true,
    },
  })
  const { code } = swcTransformSync(content, false, opts)

  return code
}

test('swc.transformSync function from native code', (t) => {
  const code = compile(
    'pages/index/index.js',
    `
  import './index.css';

  function joinPath(name) {
    return path.join(__dirname, name);
  }

  const appPath = joinPath('__app');
  console.log('appPath: ', appPath);
  `,
  )
  console.info('\ncode: \n' + code)
  t.pass()
})
