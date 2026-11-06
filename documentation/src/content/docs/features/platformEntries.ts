import { getCollection } from "astro:content";

export const platformEntries = (await getCollection("docs"))
	.filter(
		(entry) =>
			typeof entry.id === "string" &&
			entry.id.startsWith("features/platforms/") &&
			entry.id.split("/").length === 3,
	)
	.sort((left, right) => left.data.title.localeCompare(right.data.title))
	.map((entry) => ({
		href: `/${entry.id}/`,
		title: entry.data.title,
		description: entry.data.description,
	}));
