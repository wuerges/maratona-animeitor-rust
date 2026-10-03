import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

function httpsUrl(name, value) {
  if (!value) throw new Error(`${name} must be set to a public HTTPS URL`);
  const url = new URL(value);
  if (url.protocol !== 'https:' || url.username || url.password || url.search || url.hash) {
    throw new Error(`${name} must use HTTPS without credentials, query parameters, or a fragment`);
  }
  return url.href.replace(/\/+$/, '');
}

try {
  const server = httpsUrl('ANIMEITOR_SERVER_URL', process.env.ANIMEITOR_SERVER_URL);
  const config = {
    api_prefix: `${server}/api`,
    photo_prefix: httpsUrl('ANIMEITOR_PHOTO_PREFIX', process.env.ANIMEITOR_PHOTO_PREFIX || `${server}/photos`),
    sound_prefix: httpsUrl('ANIMEITOR_SOUND_PREFIX', process.env.ANIMEITOR_SOUND_PREFIX || `${server}/sounds`),
  };
  const output = process.argv[2];
  if (!output) throw new Error('Usage: node scripts/vercel-config.mjs OUTPUT_FILE');
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(config, null, 2)}\n`);
} catch (error) {
  console.error(`Vercel configuration: ${error.message}`);
  process.exitCode = 1;
}
