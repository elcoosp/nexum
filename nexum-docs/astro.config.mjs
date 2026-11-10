import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import astroTakumi from 'astro-takumi';
import fs from 'node:fs';
import { makeRender } from './src/og-render.mjs';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';

const __logoB64 = fs.readFileSync("./src/assets/logo.png").toString("base64");
const __render = makeRender({
  logoDataUrl: "data:image/png;base64," + __logoB64,
  accentHex: "#26408c",
  accentHexA12: "#26408c1a",
  domain: "nexum.vercel.app",
  projectName: "Nexum",
});

export default defineConfig({
	vite: {
		ssr: { external: ["jsdom", "astro-takumi"] },
	},
        // Deploy-specific: update `site` when the documentation moves to its final
        // origin (see docs/decisions/0007-site-url.md). Required by Starlight 0.42's
        // built-in sitemap; `base` is deliberately left unset (root deployment).
        site: 'https://nexum.vercel.app',
        integrations: [
                starlight({
			routeMiddleware: "./src/routeData.ts",
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark'],
                        },
                        plugins: [starlightLinksValidator(), starlightImageZoom(), starlightHeadingBadges()],
                        title: 'Nexum',
                        description:
                                'Framework-agnostic deep linking for Rust — one unified API for custom URL schemes and deep-link events across Windows, macOS, Linux, Android and iOS.',

                        logo: { src: './src/assets/logo.png', replacesTitle: false },
                        favicon: '/favicon.png',
                        head: [
                                { tag: 'link', attrs: { rel: 'icon', type: 'image/png', href: '/favicon.png' } },
                                { tag: 'link', attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' } },
                        ],
                        customCss: ['./src/styles/custom.css'],
                        social: [
                                {
                                        icon: 'github',
                                        label: 'GitHub',
                                        href: 'https://github.com/elcoosp/nexum',
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/nexum/edit/main/docs-site/src/content/docs/',
                        },
                        defaultLocale: 'root',
                        locales: {
                                root: {
                                        label: 'English',
                                        lang: 'en',
                                },
                        },
                        sidebar: [
                                { label: 'Overview', link: '/' },
                                {
                                        label: 'Tutorials',
                                        items: [{ slug: 'tutorials/getting-started' }],
                                },
                                {
                                        label: 'How-to Guides',
                                        items: [
                                                { slug: 'guides/installation' },
                                                { slug: 'guides/choosing-an-adapter' },
                                                { slug: 'guides/macos-setup' },
                                                { slug: 'guides/windows-setup' },
                                                { slug: 'guides/linux-setup' },
                                                { slug: 'guides/single-instance' },
                                                { slug: 'guides/bounded-hub' },
                                                { slug: 'guides/async-consumption' },
                                                { slug: 'guides/mobile-deep-links' },
                                        ],
                                },
                                {
                                        label: 'Reference',
                                        items: [
                                                { slug: 'reference/nexum' },
                                                { slug: 'reference/core' },
                                                { slug: 'reference/errors' },
                                                { slug: 'reference/platform' },
                                                { slug: 'reference/macros' },
                                                { slug: 'reference/adapters/floem' },
                                                { slug: 'reference/adapters/xilem' },
                                                { slug: 'reference/adapters/dioxus' },
                                                { slug: 'reference/adapters/leptos' },
                                                { slug: 'reference/adapters/gpui' },
                                        ],
                                },
                                {
                                        label: 'Explanation',
                                        items: [
                                                { slug: 'explanation/architecture' },
                                                { slug: 'explanation/event-sources' },
                                                { slug: 'explanation/design-principles' },
                                        ],
                                },
                        ],
                }),
        
		astroTakumi({
			options: {
				domBackend: "linkedom",
				fonts: [fs.readFileSync("./node_modules/@fontsource/roboto/files/roboto-latin-400-normal.woff2")],
				width: 1200,
				height: 630,
				format: "png",
			},
			render: __render,
		}),
	],
});
