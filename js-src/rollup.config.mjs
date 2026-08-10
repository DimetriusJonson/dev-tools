import { nodeResolve } from '@rollup/plugin-node-resolve';

export default {
  input: 'index.js',
  output: {
    file: '../public/codemirror.bundle.js',
    format: 'iife',
    name: 'CodeMirror',
    sourcemap: false
  },
  plugins: [nodeResolve()]
};
