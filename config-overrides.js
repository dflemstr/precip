module.exports = {
  webpack: function (config, env) {
    config.module.rules.unshift({
      test: /\.(jpe?g|png)$/i,
      loader: 'responsive-loader',
      options: {
        adapter: require('responsive-loader/sharp')
      }
    })
    return config
  },
  jest: function (config) {
    config.moduleNameMapper['\\.(jpe?g|png)(\\?|$)'] = '<rootDir>/src/__mocks__/responsiveLoaderMock.js'
    return config
  }
}
