import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { existsSync, readdirSync } from "fs";
import { join } from "path";
import starlightLatestVersion from "starlight-latest-version";
import starlightLinksValidator from "starlight-links-validator";
import starlightSidebarTopics from "starlight-sidebar-topics";
import starlightTagsPlugin from "starlight-tags";
import starlightSiteGraph from 'starlight-site-graph'

// Helper function to convert snake_case to Title Case
function formatLabel(str) {
	return str
		.split("_")
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(" ");
}

// Generate examples sidebar items by reading the examples directory
function getExamplesItems() {
	const examplesDir = join(process.cwd(), "src/content/docs/examples");
	const entries = readdirSync(examplesDir, { withFileTypes: true });

	return entries
		.filter((entry) => entry.isDirectory())
		.map((entry) => entry.name)
		.filter((dir) => existsSync(join(examplesDir, dir, "index.md")))
		.sort()
		.map((dir) => `examples/${dir}`);
}

export default defineConfig({
	site: "https://ubihome.github.io",
	outDir: "./site",
	prefetch: true,
	integrations: [
		starlight({
			title: "UbiHome",
			description:
				"UbiHome is a single executable that allows you to integrate any device running an OS into your smart home.",
			favicon: "/assets/favicon.png",
			logo: {
				src: "./src/content/docs/assets/logo.png",
				alt: "UbiHome",
			},
			editLink: {
				baseUrl:
					"https://github.com/UbiHome/UbiHome/edit/main/documentation/src/content/docs/",
			},
			social: [
				{
					icon: "github",
					label: "GitHub",
					href: "https://github.com/UbiHome/UbiHome",
				},
			],
			components: {
				PageTitle: "./src/components/PageTitleOverride.astro",
			},
			plugins: [
				starlightTagsPlugin({
					configPath: "tags.yml",
					tagsPagesPrefix: "tags",
					tagsIndexSlug: "tags",
					onInlineTagsNotFound: "error",
					sidebar: {
						enabled: true,
						position: "bottom",
						collapsed: true,
						showCount: true,
						showViewAllLink: true,
					},
				}),
				// TODO: https://starlight-showcases.vercel.app/components/text/
				// TODO: https://starlight-changelogs.netlify.app/providers/github/
				// TODO: https://frostybee.github.io/starlight-announcement/
				starlightSidebarTopics(
					[
						// https://starlight-sidebar-topics.netlify.app/docs/getting-started/
						{
							label: "Documentation",
							link: "/",
							icon: "open-book",
							items: [
								{
									label: "Home",
									items: [
										{ label: "Getting started", link: "/getting_started/" },
									],
								},
								{ label: "Roadmap", link: "/roadmap/" },
								{ label: "Help", items: [
										{ autogenerate: { directory: "help/" } },
                ] },
							],
						},
						{
							label: "Features",
							link: "/features",
							icon: "rocket",
							items: [
								{ label: "Overview", link: "/features/" },
								{
									label: "Components",
									items: [
										{ autogenerate: { directory: "features/components" } },
									],
								},
								{
									label: "Platforms",
									items: [
										{
											label: "Connectivity",
											items: [
												{
													autogenerate: { directory: "features/connectivity" },
												},
											],
										},
										{ autogenerate: { directory: "features/platforms" } },
									],
								},
								{
									label: "Utilities",
									items: [
										{ autogenerate: { directory: "features/utilities" } },
									],
								},
							],
						},
						{
							label: "Examples",
							link: "/examples/",
							icon: "information",
							items: [{ label: "Overview", items: getExamplesItems() }],
						},
					],
					{
						exclude: ["/tags", "/tags/**"],
					},
				),
				starlightLinksValidator({
					exclude: ["/tags/", "/tags/**"],
				}),
        // https://starlight-latest-version.trueberryless.org/
				starlightLatestVersion({
					source: {
						type: "github",
						slug: "UbiHome/UbiHome",
					},
				}),
        // https://fevol.github.io/starlight-site-graph/configuration/
				starlightSiteGraph()
			],
		}),
	],
});
