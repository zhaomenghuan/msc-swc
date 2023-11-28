import test from 'ava'

import { add } from '../index'

test('sync function from native code', (t) => {
  t.is(add(1, 2), 3)
})
