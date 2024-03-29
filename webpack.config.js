const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const Dotenv = require('dotenv-webpack');
const dist = path.resolve(__dirname, "dist");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: "./ts/bootstrap.js",
  output: {
    path: dist,
    filename: "[contenthash].js"
  },
  module: {
    rules: [
      { 
        test: /\.tsx?$/, 
        loader: "ts-loader" 
      },
      { 
        enforce: "pre", 
        test: /\.js$/, 
        loader: "source-map-loader" 
      },
      {
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },
    ]
  },
  resolve: {
    extensions: [".ts", ".tsx", ".js", ".json", ".wasm", ".css"]
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new Dotenv(),

    new HtmlWebpackPlugin({
      template: 'index.html'
    }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "client")
    }),
  ]
};
