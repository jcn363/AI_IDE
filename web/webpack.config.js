const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');

module.exports = (env, argv) => {
  const isProduction = argv.mode === 'production';
  const isDevelopment = !isProduction;
  const shouldAnalyze = argv.analyze;

  return {
    entry: './main.js',
    mode: isProduction ? 'production' : 'development',
    devtool: isDevelopment ? 'eval-source-map' : 'source-map',
    
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: isProduction ? 'assets/js/[name].[contenthash].js' : '[name].js',
      chunkFilename: isProduction ? 'assets/js/[name].[contenthash].js' : '[name].chunk.js',
      assetModuleFilename: (pathData) => {
        const filepath = path.dirname(pathData.filename).split('/').slice(1).join('/');
        if (/\.(png|jpe?g|gif|svg|webp|avif)$/i.test(pathData.filename)) {
          return `assets/images/[name].[hash][ext]`;
        }
        if (/\.(woff|woff2|eot|ttf|otf)$/i.test(pathData.filename)) {
          return `assets/fonts/[name].[hash][ext]`;
        }
        return `assets/[name].[hash][ext]`;
      },
      clean: true,
      publicPath: '/',
    },

    resolve: {
      extensions: ['.ts', '.tsx', '.js', '.jsx', '.json'],
      alias: {
        '@': path.resolve(__dirname, 'src'),
      },
      fallback: {
        "path": require.resolve("path-browserify"),
        "os": require.resolve("os-browserify/browser"),
        "crypto": require.resolve("crypto-browserify"),
        "buffer": require.resolve("buffer"),
        "process": require.resolve("process/browser"),
        "stream": require.resolve("stream-browserify"),
        "util": require.resolve("util"),
        "fs": false,
        "net": false,
        "tls": false,
      }
    },

    module: {
      rules: [
        {
          test: /\.(ts|tsx|js|jsx)$/,
          exclude: /node_modules/,
          use: {
            loader: 'babel-loader',
            options: {
              presets: [
                ['@babel/preset-env', { targets: 'defaults' }],
                ['@babel/preset-react', { runtime: 'automatic' }],
                '@babel/preset-typescript'
              ],
              plugins: [
                ['@babel/plugin-transform-react-jsx', { 
                  runtime: 'automatic',
                  importSource: '@emotion/react', 
                }],
              ],
            }
          }
        },
        {
          test: /\.css$/,
          use: [
            isDevelopment ? 'style-loader' : MiniCssExtractPlugin.loader,
            'css-loader'
          ]
        },
        {
          test: /\.(png|jpe?g|gif|svg|webp|avif)$/i,
          type: 'asset/resource',
        },
        {
          test: /\.(woff|woff2|eot|ttf|otf)$/i,
          type: 'asset/resource',
        },
        {
          test: /\.wasm$/,
          type: 'asset/resource',
        }
      ]
    },

    plugins: [
      new HtmlWebpackPlugin({
        template: './index.html',
        inject: 'body',
        scriptLoading: 'module',
      }),
      
      ...(isProduction ? [
        new MiniCssExtractPlugin({
          filename: 'assets/css/[name].[contenthash].css',
          chunkFilename: 'assets/css/[name].[contenthash].css',
        })
      ] : []),

      ...(shouldAnalyze ? [
        new BundleAnalyzerPlugin({
          analyzerMode: 'static',
          openAnalyzer: true,
          reportFilename: 'bundle-analysis.html',
        })
      ] : []),

      new (require('webpack')).ProvidePlugin({
        Buffer: ['buffer', 'Buffer'],
        process: 'process/browser',
      }),

      new (require('webpack')).DefinePlugin({
        'process.env': JSON.stringify({}),
        global: 'globalThis',
        __REACT_CONCURRENT_MODE: true,
        __REACT_DEVTOOLS_GLOBAL_HOOK__: false,
      }),
    ],

    optimization: {
      splitChunks: {
        chunks: 'all',
        cacheGroups: {
          monaco: {
            test: /[\\/]node_modules[\\/]monaco-editor/,
            name: 'monaco',
            chunks: 'all',
            priority: 30,
          },
          reactVendor: {
            test: /[\\/]node_modules[\\/](react-dom|react-router-dom)[\\/]/,
            name: 'react-vendor',
            chunks: 'all',
            priority: 25,
          },
          mui: {
            test: /[\\/]node_modules[\\/]@mui[\\/]/,
            name: 'mui',
            chunks: 'all',
            priority: 20,
          },
          three: {
            test: /[\\/]node_modules[\\/]three[\\/]/,
            name: 'three',
            chunks: 'all',
            priority: 15,
          },
          redux: {
            test: /[\\/]node_modules[\\/](@reduxjs|react-redux)[\\/]/,
            name: 'redux',
            chunks: 'all',
            priority: 10,
          },
          vendor: {
            test: /[\\/]node_modules[\\/]/,
            name: 'vendor',
            chunks: 'all',
            priority: 5,
          },
        },
      },
      runtimeChunk: 'single',
    },

    devServer: {
      port: 8080,
      hot: true,
      open: true,
      historyApiFallback: true,
      static: {
        directory: path.join(__dirname, 'public'),
      },
      headers: {
        'Cross-Origin-Embedder-Policy': 'credentialless',
        'Cross-Origin-Opener-Policy': 'same-origin',
      },
    },

    performance: {
      hints: isProduction ? 'warning' : false,
      maxAssetSize: 500000,
      maxEntrypointSize: 500000,
    },
  };
};