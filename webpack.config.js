const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
  entry: "./src/app/index.ts",
  resolve: {
    extensions: [".ts", ".js"],
  },
  output: {
    filename: "main.js",
    path: path.resolve(__dirname, "public"),
    clean: true,
  },
  mode: "development",
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true,
  },
  module: {
    rules: [{ test: /\.ts$/, use: "ts-loader" }],
  },
  plugins: [
    new CopyWebpackPlugin({
        patterns: [
            { from: "./src/app/index.html" },
            { from: "./src/app/style.css" },
            { from: "./src/app/github-logo.png" },
        ],
    }),
  ],
};
