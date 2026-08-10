import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import terser from '@rollup/plugin-terser';

export default {
  input: 'index.js',
  output: {
    file: '../app/nodejs/highlight.min.js',
    format: 'esm', // Or 'iife' for direct browser script tags
    sourcemap: false
  },
  plugins: [
    nodeResolve({
      browser: true // Optimizes resolution for browser environments
    }),
    commonjs(), // Converts CommonJS modules to ES modules if necessary
    terser(),
  ]
};
