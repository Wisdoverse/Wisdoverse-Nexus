import fs from 'node:fs'
import path from 'node:path'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const ts = createRequire(path.join(root, 'apps/web/package.json'))('typescript')
const layers = ['app', 'pages', 'widgets', 'features', 'entities', 'shared']
const sliced = new Set(['widgets', 'features', 'entities'])

export function checkFrontend(sourcePath, text) {
  const sourceParts = sourcePath.split('/')
  const sourceLayer = sourceParts[0]
  const sourceSlice = sliced.has(sourceLayer) ? sourceParts[1] : null
  const ast = ts.createSourceFile(sourcePath, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX)
  const violations = []
  function inspect(specifier, position) {
    if (!specifier.startsWith('.')) return
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(sourcePath), specifier))
    const parts = target.split('/')
    const targetLayer = parts[0]
    if (!layers.includes(targetLayer)) return
    const targetSlice = sliced.has(targetLayer) ? parts[1] : null
    const internal = sourceLayer === targetLayer && (!sourceSlice || sourceSlice === targetSlice)
    let rule
    if (layers.includes(sourceLayer) && !internal && layers.indexOf(targetLayer) <= layers.indexOf(sourceLayer)) {
      rule = 'imports must point to a lower FSD layer; sibling slices cannot import each other'
    } else if (targetSlice && !internal && !(parts.length === 2 || (parts.length === 3 && /^index(?:\.[jt]sx?)?$/.test(parts[2])))) {
      rule = 'cross-slice imports must use the slice public index'
    }
    if (rule) violations.push({ line: ast.getLineAndCharacterOfPosition(position).line + 1, rule, specifier })
  }
  function visit(node) {
    if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) && node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier)) {
      inspect(node.moduleSpecifier.text, node.getStart(ast))
    } else if (ts.isCallExpression(node) && node.arguments.length && ts.isStringLiteral(node.arguments[0]) &&
      (node.expression.kind === ts.SyntaxKind.ImportKeyword || (ts.isIdentifier(node.expression) && node.expression.text === 'require'))) {
      inspect(node.arguments[0].text, node.getStart(ast))
    } else if (ts.isImportTypeNode(node) && ts.isLiteralTypeNode(node.argument) && ts.isStringLiteral(node.argument.literal)) {
      inspect(node.argument.literal.text, node.getStart(ast))
    }
    ts.forEachChild(node, visit)
  }
  visit(ast)
  return violations
}

export function checkDomain(text) {
  // Domain files have no transport/storage runtime dependencies, including fully qualified calls.
  const withoutComments = text.replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*/g, (value) => value.replace(/[^\n]/g, ' '))
  const forbidden = /\b(?:axum|sqlx|reqwest|tokio|tower_http|tower)::|\bcrate::(?:auth|connection|router|storage|db|handlers|server|indexing|metrics|middleware|observability)::|\bsuper::(?:application|infrastructure|handlers)::/g
  return [...withoutComments.matchAll(forbidden)].map((match) => ({
    line: withoutComments.slice(0, match.index).split('\n').length,
    rule: 'DDD domain must be independent of transport, persistence and runtime adapters',
    specifier: match[0],
  }))
}

function files(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const name = path.join(directory, entry.name)
    return entry.isDirectory() ? files(name) : [name]
  })
}

function main() {
  const findings = []
  let frontendCount = 0
  let domainCount = 0
  for (const app of ['web', 'mobile']) {
    const directory = path.join(root, 'apps', app, 'src')
    for (const filename of files(directory)) {
      if (!/\.[jt]sx?$/.test(filename) || /(?:\/__tests__\/|\/test\/|\.(?:test|spec)\.[jt]sx?$|\.d\.ts$)/.test(filename)) continue
      frontendCount++
      findings.push(...checkFrontend(path.relative(directory, filename).split(path.sep).join('/'), fs.readFileSync(filename, 'utf8'))
        .map((finding) => ({ filename: path.relative(root, filename), ...finding })))
    }
  }
  for (const filename of files(path.join(root, 'crates/nexis-gateway/src'))) {
    if (!filename.endsWith('/domain.rs') && !filename.includes('/domain/')) continue
    domainCount++
    findings.push(...checkDomain(fs.readFileSync(filename, 'utf8')).map((finding) => ({ filename: path.relative(root, filename), ...finding })))
  }
  for (const finding of findings) console.error(`${finding.filename}:${finding.line}: ${finding.rule} (${finding.specifier})`)
  if (findings.length) process.exitCode = 1
  else console.log(`Architecture checks passed: ${frontendCount} frontend modules, ${domainCount} gateway domain modules.`)
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main()
