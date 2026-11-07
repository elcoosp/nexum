import { defineCollection } from 'astro:content';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';
import { z } from 'astro/zod';

export const collections = {
        docs: defineCollection({
                loader: docsLoader(),
                schema: docsSchema({
                        extend: z.object({
                                // Diátaxis page type: tutorial, how-to, reference, explanation —
                                // plus 'overview' for the landing hub page.
                                type: z
                                        .enum(['tutorial', 'how-to', 'reference', 'explanation', 'overview'])
                                        .optional(),
                        }),
                }),
        }),
        i18n: defineCollection({
                loader: i18nLoader(),
                schema: i18nSchema(),
        }),
};
