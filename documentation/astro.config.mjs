import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightLatestVersion from "starlight-latest-version";
import starlightLinksValidator from "starlight-links-validator";
import starlightSidebarTopics from "starlight-sidebar-topics";
import starlightSiteGraph from "starlight-site-graph";
import starlightTagsPlugin from "starlight-tags";
import umami from "@yeskunall/astro-umami";

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
				baseUrl: "https://github.com/UbiHome/UbiHome/edit/main/documentation/",
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
				PageSidebar: "./src/components/PageSidebarOverride.astro",
			},
			plugins: [
				// TODO: https://starlight-changelogs.netlify.app/providers/github/
				// TODO: https://frostybee.github.io/starlight-announcement/
				// https://frostybee.github.io/starlight-tags/
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
				starlightSidebarTopics(
					[
						// https://starlight-sidebar-topics.netlify.app/docs/getting-started/
						{
							label: "Documentation",
							link: "/",
							icon: "open-book",
							items: [
								{
									label: "Getting started",
									link: "/getting_started/",
								},
								{ label: "Commands", link: "/commands/" },
								{ label: "Roadmap", link: "/roadmap/" },
								{
									label: "Help",
									items: [{ autogenerate: { directory: "help/" } }],
								},
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
										{
											label: "Entities",
											collapsed: true,
											items: [
												{
													autogenerate: {
														directory: "features/entities",
													},
												},
											],
										},
									],
								},
								{
									label: "Platforms",
									items: [
										{
											label: "Connectivity and Platforms",
											link: "/features/platforms/",
										},
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
							],
						},
						{
							label: "Examples",
							link: "/examples/",
							icon: "puzzle",
							items: [{ label: "Overview", items: getExamplesItems() }],
						},
					],
					{
						exclude: ["/tags", "/tags/**"],
					},
				),
				// https://starlight-latest-version.trueberryless.org/
				starlightLatestVersion({
					source: {
						type: "github",
						slug: "UbiHome/UbiHome",
					},
				}),
				// https://fevol.github.io/starlight-site-graph/configuration/
				starlightSiteGraph(),
				starlightLinksValidator({
					exclude: ["/tags/", "/tags/**"],
				}),
			],
		}),
		umami({
			endpointUrl: "https://analytics.aquiver.de/",
			id: "47376340-dcba-422e-b906-a453eea1ede1",
		}),
	],
	// redirects: {
	// 	"/features/components/[slug]": "/features/entities/[slug]",
	// },
});
