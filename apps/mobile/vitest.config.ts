import path from 'node:path'

export default {
  root: path.resolve(__dirname),
  test: {
    environment: 'node',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.ts'],
  },
}
