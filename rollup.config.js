import bucklescript from 'rollup-plugin-bucklescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';

export default {
  input: 'ui/App.re',
  output: {
    file: 'dist/app.js',
    format: 'iife',
  },
  plugins: [
    bucklescript(),
    resolve(),
    commonjs({
      namedExports: {
        'node_modules/react/index.js': ['createElement', 'useState'],
        'node_modules/react-dom/index.js': ['render'],
      },
    }),
  ],
}
