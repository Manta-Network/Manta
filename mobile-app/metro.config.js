const { getDefaultConfig } = require('expo/metro-config');
const { withNativeWind } = require('nativewind/metro');

const config = getDefaultConfig(__dirname);

// Add support for .cjs and .mjs files (needed for Polkadot.js)
config.resolver.sourceExts = [...config.resolver.sourceExts, 'cjs', 'mjs'];

// Node polyfills for Polkadot.js
config.resolver.extraNodeModules = {
  ...config.resolver.extraNodeModules,
  stream: require.resolve('readable-stream'),
  buffer: require.resolve('buffer'),
  process: require.resolve('process/browser'),
};

module.exports = withNativeWind(config, { input: './global.css' });
