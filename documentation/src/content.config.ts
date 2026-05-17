import { defineCollection } from "astro:content";
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";
import { starlightTagsExtension } from "starlight-tags/schema";
import { pageSiteGraphSchema } from "starlight-site-graph/schema";

export const collections = {
	docs: defineCollection({
		loader: docsLoader(),
		schema: docsSchema({
			extend: starlightTagsExtension.and(pageSiteGraphSchema),
		}),
	}),
};
