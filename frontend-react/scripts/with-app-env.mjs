#!/usr/bin/env node
/**
 * Development environment setup
 * Loads and validates environment variables before running Vite
 */

import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import { config } from 'dotenv'
import { spawn } from 'child_process'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const rootDir = join(__dirname, '..')

// Load .env.local or .env
config({ path: join(rootDir, '.env.local') })
config({ path: join(rootDir, '.env') })

// Validate required environment variables
const required = ['VITE_API_URL']
const missing = required.filter(key => !process.env[key])

if (missing.length > 0) {
  console.warn(`\n⚠️  Missing environment variables: ${missing.join(', ')}`)
  console.warn(`Please create .env.local or set these variables.\n`)
  
  // Set defaults
  if (!process.env.VITE_API_URL) {
    process.env.VITE_API_URL = 'http://localhost:3000'
    console.log(`Using default VITE_API_URL=${process.env.VITE_API_URL}`)
  }
}

// Run the actual command
const [, , ...args] = process.argv
const child = spawn('node', args, {
  stdio: 'inherit',
  env: process.env,
})

child.on('exit', (code) => {
  process.exit(code || 0)
})
