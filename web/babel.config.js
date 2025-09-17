module.exports = {
  presets: [
    ['@babel/preset-env', { 
      targets: 'defaults',
      modules: false // Let webpack handle modules
    }],
    ['@babel/preset-react', { 
      runtime: 'automatic' 
    }],
    '@babel/preset-typescript'
  ],
  plugins: [
    ['@babel/plugin-transform-react-jsx', { 
      runtime: 'automatic',
      importSource: '@emotion/react', 
    }],
  ],
  env: {
    test: {
      presets: [
        ['@babel/preset-env', { 
          targets: { node: 'current' },
          modules: 'commonjs' // Use CommonJS for Jest
        }],
        ['@babel/preset-react', { 
          runtime: 'automatic' 
        }],
        '@babel/preset-typescript'
      ]
    }
  }
};