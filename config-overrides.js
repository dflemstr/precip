module.exports = function override (config, env) {
  config.module.rules.unshift({
    test: /\.(jpe?g|png)$/i,
    loader: 'responsive-loader',
    options: {
      adapter: require('responsive-loader/sharp')
    }
  })
  return config
}
