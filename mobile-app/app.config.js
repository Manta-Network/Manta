// app.config.js - Extends app.json with dynamic configuration
module.exports = ({ config }) => ({
  ...config,
  plugins: [
    ...(config.plugins || []),
    'expo-router',
    'expo-secure-store',
    'expo-asset',
    'expo-font',
    'expo-web-browser'
  ],
  android: {
    ...(config.android || {}),
    package: "network.chml.wallet",
    adaptiveIcon: {
      foregroundImage: "./assets/images/icon.png",
      backgroundColor: "#22B958"
    }
  },
  extra: {
    ...(config.extra || {}),
    rpcEndpoint: "ws://64.23.233.36:9944",
    rpcHttpEndpoint: "http://64.23.233.36:9933",
    tokenSymbol: "CHML",
    tokenDecimals: 18,
    eas: {
      projectId: "df137d68-ff1c-4b64-9267-857b1a904023"
    }
  }
});
