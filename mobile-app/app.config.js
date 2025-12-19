// app.config.js - Extends app.json with dynamic configuration
module.exports = ({ config }) => ({
  ...config,
  // Override or extend app.json values here if needed
  extra: {
    ...config.extra,
    // EAS projectId is required
    eas: {
      projectId: "df137d68-ff1c-4b64-9267-857b1a904023"
    }
  }
});
