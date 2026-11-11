import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';


export default await defineDocsConfig({
  llmsTxt: {},
  changelog: { repo: 'elcoosp/nexum' },
  site: 'https://nexum.vercel.app',
  domain: 'nexum.vercel.app',
  title: "Nexum",
  description: "Framework-agnostic deep linking for Rust — one unified API for custom URL schemes and deep-link events across Windows, macOS, Linux, Android and iOS.",
  sidebar: [],
  extraStarlight: {
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark']
                        },
                        plugins: [starlightLinksValidator(), starlightImageZoom(), starlightHeadingBadges()],
                        title: 'Nexum',
                        description:
                                'Framework-agnostic deep linking for Rust — one unified API for custom URL schemes and deep-link events across Windows, macOS, Linux, Android and iOS.',
                        favicon: '/favicon.png',
                        head: [
                                { tag: 'link', attrs: { rel: 'icon', type: 'image/png', href: '/favicon.png' } },
                                { tag: 'link', attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' } },
                        ],
                        social: [
                                {
                                        icon: 'github',
                                        label: 'GitHub',
                                        href: 'https://github.com/elcoosp/nexum'
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/nexum/edit/main/docs-site/src/content/docs/'
                        },
                        defaultLocale: 'root',
                        locales: {
                                root: {
                                        label: 'English',
                                        lang: 'en'
                                }
                        },
                        sidebar: [
                                { label: 'Overview', link: '/' },
                                {
                                        label: 'Tutorials',
                                        items: [{ slug: 'tutorials/getting-started' }]
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
                                        ]
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
                                        ]
                                },
                                {
                                        label: 'Explanation',
                                        items: [
                                                { slug: 'explanation/architecture' },
                                                { slug: 'explanation/event-sources' },
                                                { slug: 'explanation/design-principles' },
                                        ]
                                },
                        ]
                },
});
