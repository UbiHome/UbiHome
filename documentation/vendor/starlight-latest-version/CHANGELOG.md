# starlight-latest-version

## 0.7.0

### Minor Changes

- [#68](https://github.com/trueberryless-org/starlight-latest-version/pull/68) [`d88a13f`](https://github.com/trueberryless-org/starlight-latest-version/commit/d88a13ff2225b8889421d87485f0b41598b729bb) Thanks [@trueberryless](https://github.com/trueberryless)! - Adds support for Astro v6, drops support for Astro v5.

  ⚠️ **BREAKING CHANGE:** The minimum supported version of Starlight is now `0.38.0`.

  Please follow the [upgrade guide](https://github.com/withastro/starlight/releases/tag/%40astrojs%2Fstarlight%400.38.0) to update your project.

## 0.6.1

### Patch Changes

- [#64](https://github.com/trueberryless-org/starlight-latest-version/pull/64) [`187cf39`](https://github.com/trueberryless-org/starlight-latest-version/commit/187cf39b5700cb6618a942b23edad659a608afb1) Thanks [@trueberryless](https://github.com/trueberryless)! - Clean up package metadata.

## 0.6.0

### Minor Changes

- [#58](https://github.com/trueberryless-org/starlight-latest-version/pull/58) [`f4776fb`](https://github.com/trueberryless-org/starlight-latest-version/commit/f4776fb8602ca30bbf9de592fa062b4e261a5225) Thanks [@trueberryless](https://github.com/trueberryless)! - Rename package from `starlight-plugin-show-latest-version` to `starlight-latest-version`.

## 0.5.1

### Patch Changes

- [#52](https://github.com/trueberryless-org/starlight-latest-version/pull/52) [`1152b13`](https://github.com/trueberryless-org/starlight-latest-version/commit/1152b137cc1fec970fb604213bb7a71b7fff2597) Thanks [@trueberryless-org](https://github.com/apps/trueberryless-org)! - Setups trusted publishing using OpenID Connect (OIDC) authentication — no code changes.

## 0.5.0

### Minor Changes

- [#32](https://github.com/trueberryless-org/starlight-latest-version/pull/32) [`9bdad60`](https://github.com/trueberryless-org/starlight-latest-version/commit/9bdad60c0f257bb5889c5ca9c0700d2e5c9dcc9b) Thanks [@trueberryless](https://github.com/trueberryless)! - ⚠️ **BREAKING CHANGE:** If you set [`showInSiteTitle`](https://starlight-latest-version.trueberryless.org/configuration#showinsitetitle) to `"deferred"`, you will have to [enable experimental sessions](https://docs.astro.build/en/reference/experimental-flags/sessions/#enabling-experimental-sessions) because the version will be cached for an hour.

## 0.4.0

### Minor Changes

- [#29](https://github.com/trueberryless-org/starlight-latest-version/pull/29) [`3656c8f`](https://github.com/trueberryless-org/starlight-latest-version/commit/3656c8f9b35ca3cf4cade1791098301020bc117c) Thanks [@trueberryless](https://github.com/trueberryless)! - ⚠️ **BREAKING CHANGE:** The minimum supported version of Starlight is now version `0.32.0`.

  Please use the `@astrojs/upgrade` command to upgrade your project:

  ```sh
  npx @astrojs/upgrade
  ```

## 0.3.1

### Patch Changes

- [#26](https://github.com/trueberryless-org/starlight-latest-version/pull/26) [`38e0418`](https://github.com/trueberryless-org/starlight-latest-version/commit/38e04186f90a4e575dde53339e290098574a2db4) Thanks [@trueberryless](https://github.com/trueberryless)! - Prevents the header title from being translated by automatic translation systems.

## 0.3.0

Special thanks to [@HiDeoo](https://github.com/HiDeoo) for their valuable feedback, which helped shape many of the improvements in this release.

### Minor Changes

- [#16](https://github.com/trueberryless-org/starlight-latest-version/pull/16) [`3de9964`](https://github.com/trueberryless-org/starlight-latest-version/commit/3de9964f4860928c42754c94e8be1c246b1cc674) Thanks [@trueberryless](https://github.com/trueberryless)! - Use [`server:defer`](https://docs.astro.build/en/reference/directives-reference/#serverdefer) in the `SiteTitle.astro` override component if [`showInSiteTitle`](https://starlight-latest-version.trueberryless.org/configuration/#showinsitetitle) is set to `"deferred"`.

  ⚠️ **BREAKING CHANGE:** You now have to use some [server adapter](https://docs.astro.build/en/guides/on-demand-rendering/#server-adapters) if you set [`showInSiteTitle`](https://starlight-latest-version.trueberryless.org/configuration/#showinsitetitle) to `"deferred"`.

  If you set the [`showInSiteTitle` configuration option](https://starlight-latest-version.trueberryless.org/configuration/#showinsitetitle) to `"deferred"`, you have to add a [server adapter](https://docs.astro.build/en/guides/on-demand-rendering/#server-adapters) because the plugin override now uses [`server:defer`](https://docs.astro.build/en/reference/directives-reference/#serverdefer) in order to fetch the latest version on-demand.

  Read more about Server Islands in [this blog post](https://astro.build/blog/future-of-astro-server-islands/) or the [Astro documentation](https://docs.astro.build/en/guides/server-islands/).

- [#14](https://github.com/trueberryless-org/starlight-latest-version/pull/14) [`8b0c933`](https://github.com/trueberryless-org/starlight-latest-version/commit/8b0c933c19b1fc1ed035e85a45168c0ec1b4f3a7) Thanks [@trueberryless](https://github.com/trueberryless)! - Add version support for NPM and GitLab.

  ⚠️ **BREAKING CHANGE:** The configuration interface changed.

  Please follow the steps below to use the plugin like before or read the [documentation](https://starlight-latest-version.trueberryless.org/configuration/#source) for the newly defined API.

  Change the removed `repo` configuration to the new [`source`](https://starlight-latest-version.trueberryless.org/configuration/#source) configuration object:

  ```diff
   // astro.config.ts
   starlightLatestVersion({
  -  repo: "${slug}",
  +  source: {
  +    type: "github",
  +    slug: "${slug}",
  +  },
   }),
  ```

  Note that you can now use `"npm"`, `"github"` or `"gitlab"` as the source type. This means that the plugin can be better customised to where you publish and release your packages.

### Patch Changes

- [#16](https://github.com/trueberryless-org/starlight-latest-version/pull/16) [`3de9964`](https://github.com/trueberryless-org/starlight-latest-version/commit/3de9964f4860928c42754c94e8be1c246b1cc674) Thanks [@trueberryless](https://github.com/trueberryless)! - Make override of Starlight `SiteTitle.astro` optional. You now have to opt-in by setting the new [`showInSiteTitle`](https://starlight-latest-version.trueberryless.org/configuration/#showinsitetitle) configuration option to `"true"` because it does not get overridden by default.

- [#16](https://github.com/trueberryless-org/starlight-latest-version/pull/16) [`ae72935`](https://github.com/trueberryless-org/starlight-latest-version/commit/ae72935cbdca23c5e7d880d4f0c82c57c328e874) Thanks [@trueberryless](https://github.com/trueberryless)! - Add simple [fallback content](https://docs.astro.build/en/guides/server-islands/#server-island-fallback-content) for SiteTitle override Version Badge

## 0.2.0

### Minor Changes

- [#12](https://github.com/trueberryless-org/starlight-latest-version/pull/12) [`c4c5257`](https://github.com/trueberryless-org/starlight-latest-version/commit/c4c525794ba68fe9f33d16194c6802632f5cba77) Thanks [@trueberryless](https://github.com/trueberryless)! - Adds support for Astro v5, drops support for Astro v4.

  ⚠️ **BREAKING CHANGE:** The minimum supported version of Starlight is now `0.30.0`.

  Please follow the [upgrade guide](https://github.com/withastro/starlight/releases/tag/%40astrojs/starlight%400.30.0) to update your project.

  Note that the [`legacy.collections` flag](https://docs.astro.build/en/reference/legacy-flags/#collections) is not supported by this plugin and you should update your collections to use Astro's new Content Layer API.

### Patch Changes

- [`e29e231`](https://github.com/trueberryless-org/starlight-latest-version/commit/e29e2318eb36400ee5752017487518f07d091e31) Thanks [@trueberryless](https://github.com/trueberryless)! - Change Client JS to insert Raw HTML into div so it doesn't need rendering

## 0.1.1

### Patch Changes

- [`d84a498`](https://github.com/trueberryless-org/starlight-latest-version/commit/d84a4981b81d3c4402834028e0f96f23be4c5a4e) Thanks [@trueberryless](https://github.com/trueberryless)! - Remove virtual export functionality because it's much harder than I though, maybe not even possible.

## 0.1.0

### Minor Changes

- [`ab7b842`](https://github.com/trueberryless-org/starlight-latest-version/commit/ab7b842691b74692c513ecd5e4557112a7eccca6) Thanks [@trueberryless](https://github.com/trueberryless)! - Release first minor version. Latest version in site title and virtual module export.
