# `@msc-studio/swc`

> msc swc package with napi-rs.

## 调试

开发完毕后可通过执行`yarn build`生成`napi`构建完的`JS`产物，通过执行`yarn test`进行测试。

## 发布

修改版本号，包含根目录以及`npm/`目录下的`package.json`。
代码提交后可触发`github action`构建。

github CI 有校验，如果需要发布 npm 包，需要commit message 格式为： chore(relase): a.b.c。
