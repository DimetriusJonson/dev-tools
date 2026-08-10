import { nodeResolve } from '@rollup/plugin-node-resolve';
import terser from '@rollup/plugin-terser';

export default {
  input: 'index.js',
  output: {
    file: '../public/codemirror.min.js',
    format: 'iife',
    name: 'CodeMirror',
    sourcemap: false
  },
  plugins: [nodeResolve(), terser()]
};
