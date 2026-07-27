#!/usr/bin/env node

import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { spawn } from 'node:child_process'

const repoRoot = path.resolve(new URL('../..', import.meta.url).pathname)
const desktopPackagePath = path.join(repoRoot, 'apps/desktop/package.json')
const tauriConfigPath = path.join(repoRoot, 'apps/desktop/src-tauri/tauri.conf.json')
const cargoTomlPath = path.join(repoRoot, 'apps/desktop/src-tauri/Cargo.toml')
const cargoLockPath = path.join(repoRoot, 'apps/desktop/src-tauri/Cargo.lock')
const cachePath = path.join(repoRoot, '.git/.docs-atlas-release.json')

const args = parseArgs(process.argv.slice(2))
const bumpType = normalizeBumpType(args.bump ?? args._[0] ?? 'patch')
const dryRun = Boolean(args['dry-run'])
const remote = String(args.remote ?? 'origin')

const currentVersion = await readCurrentVersion()
const nextVersion = bumpVersion(currentVersion, bumpType)
const releaseTag = `desktop-v${nextVersion}`
const defaultCommitMessage = `release(desktop): ${releaseTag}`
const cachedState = await readCache()
const commitMessage = String(args.message ?? args.m ?? cachedState.lastCommitMessage ?? defaultCommitMessage).trim()
const tagMessage = String(args['tag-message'] ?? `Docs Atlas Desktop ${releaseTag}`).trim()

if (!commitMessage) {
  throw new Error('Commit message is empty. Pass --message "..." to continue.')
}

await ensureInsideGitRepo()
if (dryRun) {
  await ensureLocalTagDoesNotExist(releaseTag)
} else {
  await ensureTagDoesNotExist(releaseTag, remote)
}

const branchName = await gitOutput(['rev-parse', '--abbrev-ref', 'HEAD'])

if (dryRun) {
  console.log(JSON.stringify({
    currentVersion,
    nextVersion,
    bumpType,
    branchName,
    remote,
    releaseTag,
    commitMessage,
    tagMessage,
  }, null, 2))
  process.exit(0)
}

if (branchName === 'HEAD') {
  throw new Error('Detached HEAD is not supported. Checkout a branch before releasing.')
}

await writeVersionFiles(nextVersion)
await runNodeScript(path.join(repoRoot, 'scripts/release/verify-desktop-version.mjs'))
await git(['add', '-A'])
await git(['commit', '-m', commitMessage])
await git(['tag', '-a', releaseTag, '-m', tagMessage])
await git(['push', remote, branchName])
await git(['push', remote, releaseTag])
await writeCache({ lastCommitMessage: commitMessage })

console.log(`Released ${releaseTag}`)

function parseArgs(argv) {
  const parsed = { _: [] }

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]

    if (!token.startsWith('-')) {
      parsed._.push(token)
      continue
    }

    if (token.startsWith('--')) {
      const key = token.slice(2)
      const next = argv[index + 1]

      if (!next || next.startsWith('-')) {
        parsed[key] = true
        continue
      }

      parsed[key] = next
      index += 1
      continue
    }

    const shortKey = token.slice(1)
    const next = argv[index + 1]

    if (!next || next.startsWith('-')) {
      parsed[shortKey] = true
      continue
    }

    parsed[shortKey] = next
    index += 1
  }

  return parsed
}

function normalizeBumpType(value) {
  const normalized = String(value).toLowerCase()

  if (normalized === 'major' || normalized === 'minor' || normalized === 'patch') {
    return normalized
  }

  throw new Error(`Unsupported bump type: ${value}. Use major, minor, or patch.`)
}

async function readCurrentVersion() {
  const packageJson = JSON.parse(await fs.readFile(desktopPackagePath, 'utf8'))
  const version = String(packageJson.version ?? '').trim()

  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    throw new Error(`Invalid desktop version: ${version}`)
  }

  return version
}

function bumpVersion(version, bumpType) {
  const [major, minor, patch] = version.split('.').map((part) => Number.parseInt(part, 10))

  if ([major, minor, patch].some(Number.isNaN)) {
    throw new Error(`Invalid semver version: ${version}`)
  }

  if (bumpType === 'major') {
    return `${major + 1}.0.0`
  }

  if (bumpType === 'minor') {
    return `${major}.${minor + 1}.0`
  }

  return `${major}.${minor}.${patch + 1}`
}

async function writeVersionFiles(version) {
  const packageJson = JSON.parse(await fs.readFile(desktopPackagePath, 'utf8'))
  packageJson.version = version
  await fs.writeFile(desktopPackagePath, `${JSON.stringify(packageJson, null, 2)}\n`, 'utf8')

  const tauriConfig = JSON.parse(await fs.readFile(tauriConfigPath, 'utf8'))
  tauriConfig.version = version
  await fs.writeFile(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`, 'utf8')

  const cargoToml = await fs.readFile(cargoTomlPath, 'utf8')
  const nextCargoToml = cargoToml.replace(/^version\s*=\s*"([^"]+)"\s*$/m, `version = "${version}"`)

  if (nextCargoToml === cargoToml) {
    throw new Error('Unable to update version in Cargo.toml')
  }

  await fs.writeFile(cargoTomlPath, nextCargoToml, 'utf8')

  const cargoLock = await fs.readFile(cargoLockPath, 'utf8')
  const packageMarker = 'name = "docs-atlas"'
  const packageIndex = cargoLock.indexOf(packageMarker)
  if (packageIndex < 0) {
    throw new Error('Unable to find docs-atlas package in Cargo.lock')
  }

  const versionIndex = cargoLock.indexOf('version = "', packageIndex)
  if (versionIndex < 0) {
    throw new Error('Unable to find docs-atlas version in Cargo.lock')
  }

  const versionEndIndex = cargoLock.indexOf('\n', versionIndex)
  const nextCargoLock = `${cargoLock.slice(0, versionIndex)}version = "${version}"${cargoLock.slice(versionEndIndex)}`
  await fs.writeFile(cargoLockPath, nextCargoLock, 'utf8')
}

async function readCache() {
  try {
    const raw = await fs.readFile(cachePath, 'utf8')
    return JSON.parse(raw)
  } catch {
    return {}
  }
}

async function writeCache(payload) {
  await fs.writeFile(cachePath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8')
}

async function ensureInsideGitRepo() {
  await gitOutput(['rev-parse', '--is-inside-work-tree'])
}

async function ensureLocalTagDoesNotExist(tagName) {
  const localTag = await gitOutput(['tag', '--list', tagName])

  if (localTag.trim()) {
    throw new Error(`Tag already exists locally: ${tagName}`)
  }
}

async function ensureTagDoesNotExist(tagName, remoteName) {
  await ensureLocalTagDoesNotExist(tagName)

  const remoteTag = await gitOutput(['ls-remote', '--tags', remoteName, tagName])

  if (remoteTag.trim()) {
    throw new Error(`Tag already exists on ${remoteName}: ${tagName}`)
  }
}

function git(args) {
  return runCommand('git', args)
}

function gitOutput(args) {
  return runCommand('git', args, { captureStdout: true })
}

function runNodeScript(scriptPath) {
  return runCommand(process.execPath, [scriptPath])
}

function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: repoRoot,
      stdio: options.captureStdout ? ['inherit', 'pipe', 'inherit'] : 'inherit',
      env: process.env,
    })

    let stdout = ''

    if (options.captureStdout && child.stdout) {
      child.stdout.on('data', (chunk) => {
        stdout += String(chunk)
      })
    }

    child.on('error', reject)
    child.on('close', (code) => {
      if (code === 0) {
        resolve(options.captureStdout ? stdout.trim() : '')
        return
      }

      reject(new Error(`${command} ${args.join(' ')} failed with exit code ${code}`))
    })
  })
}
