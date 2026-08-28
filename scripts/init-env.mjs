import { randomBytes } from 'node:crypto';
import { writeFile } from 'node:fs/promises';
const password = randomBytes(24).toString('hex');
const key = randomBytes(32).toString('base64');
const content = `RUSTDB_DATABASE_URL=postgresql://rustdb:${password}@localhost:5432/rustdb
RUSTDB_ENCRYPTION_KEY=${key}
RUSTDB_BIND_ADDRESS=127.0.0.1
RUSTDB_PORT=8080
RUSTDB_CORS_ORIGIN=http://localhost:5173
RUSTDB_CONNECT_TIMEOUT_SECONDS=5
RUSTDB_QUERY_TIMEOUT_SECONDS=30
RUSTDB_MAX_ROWS=1000
RUST_LOG=info
POSTGRES_USER=rustdb
POSTGRES_PASSWORD=${password}
POSTGRES_DB=rustdb
`;
try {
  await writeFile(new URL('../.env', import.meta.url), content, { flag: 'wx', mode: 0o600 });
  console.log('.env created with random credentials. Keep it private and back it up securely.');
} catch (error) {
  console.error(error.code === 'EEXIST' ? '.env already exists; nothing was overwritten.' : 'Unable to create .env.');
  process.exitCode = 1;
}
