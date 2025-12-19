module.exports = function(api) {
  api.cache(true);

  return {
    presets: [
      [
        'babel-preset-expo',
        {
          jsxImportSource: 'nativewind',
          unstable_transformImportMeta: true  // Enable for Polkadot.js
        }
      ],
      'nativewind/babel'
    ],
    plugins: [
      '@babel/plugin-transform-class-static-block',
      [
        'module-resolver',
        {
          root: ['./'],
          alias: {
            '@': './',
            'tailwind.config': './tailwind.config.js'
          }
        }
      ],
      'react-native-reanimated/plugin'  // Must be last
    ]
  };
};
