import { log } from '../../utils/util';
import { config } from '../../config/index';

Page({
  data: {
    env: 'production',
  },
  onLoad() {
    log('Page onLoad, env: ', config.env);
    this.setState({
      env: config.env,
    });
  },
});
