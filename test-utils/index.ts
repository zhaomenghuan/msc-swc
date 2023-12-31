import path from 'path';
import fs from 'fs';
import { transformSync } from '../index';

export function resolveFileType(fileName: string) {
  return path.extname(fileName).substring(1);
}

export function toBuffer(t: unknown): Buffer {
  return Buffer.from(JSON.stringify(t));
}

export function compile(cwd: string, filename: string, content?: string) {
  if (!content) {
    content = fs.readFileSync(path.join(cwd, filename), 'utf-8').toString();
  }

  const fileType = resolveFileType(filename);
  const enableTypescript = fileType === 'ts' || fileType === 'tsx';
  const enableJSX = fileType === 'jsx' || fileType === 'tsx';
  const options = toBuffer({
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
  const customOptions = toBuffer({
    externalPackages: ['react'],
  });
  console.info('\n-----------------------------');
  const result = transformSync(content, options, customOptions);
  console.info('filename: ', filename);
  console.info('code: \n', result.code);
  console.info('metadata:', result.metadata);
  console.info('-----------------------------\n');
  return result;
}
