#!/usr/bin/env node
/**
 * Generate llms.txt and llms-full.txt at the site root (dist/), per the
 * AI-agent discoverability requirement.
 *
 * - llms.txt: title, one-line description and absolute URL for every docs page.
 * - llms-full.txt: the same index followed by each page's full markdown body.
 *
 * Runs as part of `pnpm build` (after `astro build`).
 * Keep SITE in sync with astro.config.mjs (docs/decisions/0007-site-url.md).
 */
import { readdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

const DIST = 'dist';
const SITE = 'https://elcoosp.github.io/nexum'; // keep in sync with astro.config.mjs `site`
const DOCS = 'src/content/docs';
const SITE_TITLE = 'Nexum';
const SITE_DESCRIPTION =
        'Framework-agnostic deep linking for Rust — register custom URL schemes and receive deep-link events with one unified API.';

async function walk(dir) {
        const entries = await readdir(dir, { withFileTypes: true });
        const files = [];
        for (const e of entries) {
                const p = path.join(dir, e.name);
                if (e.isDirectory()) files.push(...(await walk(p)));
                else if (e.name === 'index.html') files.push(p);
        }
        return files;
}

const htmlFiles = await walk(DIST);
const routes = htmlFiles
        .map((f) => {
                const rel = path.relative(DIST, path.dirname(f));
                return '/' + (rel === '.' ? '' : rel + '/');
        })
        // content pages live at the site root; skip the built-in 404
        .filter((r) => r !== '/404/')
        .sort();

const pages = [];
for (const route of routes) {
        const slug = route.replace(/^\/|\/$/g, '');
        let src = null;
        for (const ext of ['.mdx', '.md']) {
                const candidate = path.join(DOCS, slug + ext);
                try {
                        src = await readFile(candidate, 'utf8');
                        break;
                } catch {
                        /* try next extension */
                }
        }
        if (!src) continue;
        const fm = src.match(/^---\n([\s\S]*?)\n---/);
        const title = fm?.[1]?.match(/^title:\s*(.+)$/m)?.[1]?.replace(/^["']|["']$/g, '') ?? slug;
        const description =
                fm?.[1]?.match(/^description:\s*(.+)$/m)?.[1]?.replace(/^["']|["']$/g, '') ?? '';
        const body = fm ? src.slice(fm[0].length).trim() : src.trim();
        pages.push({ url: SITE + route, title, description, body });
}

const header = `# ${SITE_TITLE}\n\n> ${SITE_DESCRIPTION}\n`;

const indexLines = pages.map((p) => `- [${p.title}](${p.url}): ${p.description}`);
await writeFile(path.join(DIST, 'llms.txt'), `${header}\n## Pages\n\n${indexLines.join('\n')}\n`);

const full = [
        header,
        '',
        ...pages.flatMap((p) => [`---`, `# ${p.title}`, `URL: ${p.url}`, '', p.body, '']),
];
await writeFile(path.join(DIST, 'llms-full.txt'), full.join('\n'));

console.log(`llms.txt + llms-full.txt generated for ${pages.length} routes`);
