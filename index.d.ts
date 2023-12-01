/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export function transformSync(s: string, isModule: boolean, opts: Buffer): TransformOutput;
export interface TransformOutput {
  code: string;
  map?: string;
}
export function minifySync(code: Buffer, opts: Buffer): TransformOutput;
export interface Metadata {
  requires: Array<string>;
}
export interface SwcTransformOutput {
  code: string;
  map?: string;
  metadata: Metadata;
}
export function swcTransformSync(s: string, opts: Buffer): SwcTransformOutput;
