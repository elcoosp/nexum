import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';

export default defineConfig({
        // Deploy-specific: update `site` when the documentation moves to its final
        // origin (see docs/decisions/0007-site-url.md). Required by Starlight 0.42's
        // built-in sitemap; `base` is deliberately left unset (root deployment).
        site: 'https://elcoosp.github.io/nexum',
        integrations: [
                starlight({
                        plugins: [starlightLinksValidator(), starlightImageZoom(), starlightHeadingBadges()],
                        title: 'Nexum',
                        description:
                                'Framework-agnostic deep linking for Rust — one unified API for custom URL schemes and deep-link events across Windows, macOS, Linux, Android and iOS.',
                        logo: {
                                src: './src/assets/logo.png',
                                alt: 'Nexum',
                                replacesTitle: false,
                        },
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
        ],
});
