const path = require("path");

module.exports = {
  entry: "./src/index.js",
  output: {
    path: path.resolve(__dirname, "../client/dist"),
    filename: "bundle.js",
  },
};
