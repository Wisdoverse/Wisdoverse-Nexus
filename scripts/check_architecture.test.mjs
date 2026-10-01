import assert from 'node:assert/strict'
import test from 'node:test'
import { checkDomain, checkFrontend } from './check_architecture.mjs'

test('downward public imports and internal slice modules are allowed', () => {
  assert.deepEqual(checkFrontend('features/messages/ui.tsx', `
    import { session } from '../../entities/session'
    export { model } from './model'
    import { http } from '../../shared/api/httpClient'
  `), [])
})

test('type-only and dynamic imports cannot bypass FSD boundaries', () => {
  assert.equal(checkFrontend('features/messages/ui.tsx', `
    import type { Route } from '../../app/navigation/types'
    export { room } from '../rooms'
    const login = import('../auth')
    type Private = import('../../entities/session/sessionStore').State
  `).length, 4)
})

test('app composition uses slice public APIs', () => {
  assert.equal(checkFrontend('app/navigation/root.tsx', `
    import { Login } from '../../features/auth/TokenLoginScreen'
    const store = require('../../entities/session/sessionStore')
  `).length, 2)
  assert.deepEqual(checkFrontend('app/navigation/root.tsx', `import { Login } from '../../features/auth/index.ts'`), [])
})

test('DDD domain rejects runtime and adapter dependencies while ignoring comments', () => {
  assert.deepEqual(checkDomain('// axum:: is deliberately absent\nuse chrono::Utc;'), [])
  assert.equal(checkDomain('use axum::Json;\nuse super::infrastructure::SqlStore;\ntokio::spawn(task);\nuse crate::db::Pool;\ncrate::handlers::dispatch();').length, 5)
})
