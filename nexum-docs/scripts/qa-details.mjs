#!/usr/bin/env node
/* Capture zoomed detail shots for the eyeball pass:
   - header logo region at deviceScaleFactor 3 (light + dark) for sharpness
   - search modal with a domain query
   - a code block region (dark)
   - pagination region (light)
   Then compose a single 2x2 detail sheet. */
import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';

const BASE = 'http://localhost:4321';
const OUT = '.qa/details';
await mkdir(OUT, { recursive: true });

const browser = await chromium.launch();

async function capture({ url, theme, dsf = 2, clip, out, openSearch = false }) {
	const ctx = await browser.newContext({
		viewport: { width: 1440, height: 900 },
		deviceScaleFactor: dsf,
		reducedMotion: 'reduce',
	});
	const page = await ctx.newPage();
	await page.addInitScript((t) => {
		try {
			localStorage.setItem('starlight-theme', t);
		} catch {}
	}, theme);
	await page.goto(BASE + url, { waitUntil: 'networkidle' });
	await page.evaluate(() => document.fonts.ready);
	if (openSearch) {
		// Starlight search button — click and type
		const btn = page.locator('button[aria-label="Search"], starlight-search-button button, button[data-open-search]').first();
		await btn.click();
		await page.waitForTimeout(400);
		await page.keyboard.type('deep link');
		await page.waitForTimeout(600);
	}
	await page.screenshot({ path: out, fullPage: false, clip });
	await ctx.close();
	console.log('captured', out);
}

const headerClip = { x: 180, y: 0, width: 700, height: 110 };

await capture({ url: '/reference/core/', theme: 'light', dsf: 3, clip: headerClip, out: `${OUT}/logo-zoom-light.png` });
await capture({ url: '/reference/core/', theme: 'dark', dsf: 3, clip: headerClip, out: `${OUT}/logo-zoom-dark.png` });
await capture({ url: '/tutorials/getting-started/', theme: 'light', openSearch: true, out: `${OUT}/search-modal.png` });
// code block area on the tutorial (below the first rust snippet)
await capture({ url: '/tutorials/getting-started/', theme: 'dark', clip: { x: 280, y: 420, width: 900, height: 420 }, out: `${OUT}/code-dark.png` });
// pagination at bottom of a reference page
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, reducedMotion: 'reduce' });
const page = await ctx.newPage();
await page.goto(BASE + '/reference/platform/', { waitUntil: 'networkidle' });
const h = await page.evaluate(() => document.body.scrollHeight);
await page.evaluate((y) => window.scrollTo(0, y), h);
await page.waitForTimeout(300);
await page.screenshot({ path: `${OUT}/pagination.png` });
await ctx.close();
console.log('captured', `${OUT}/pagination.png`);

await browser.close();
