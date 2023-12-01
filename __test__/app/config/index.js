export const config =
  process.env.NODE_ENV === 'production' ? require('./app.prod.config') : require('./app.dev.config');
