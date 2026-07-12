declare module "virtual:starlight-latest-version-config" {
  const starlightLatestVersionConfig: import("./index").starlightLatestVersionConfig;
  export default starlightLatestVersionConfig;
}

declare module "virtual:starlight-latest-version" {
  /**
   * Fetch the latest version of the configured source.
   *
   * Returns the same context object used internally by the plugin's
   * components, so you can read `version`, `versionWithoutPrefix`,
   * `versionMajor` and friends programmatically.
   */
  export function getLatestVersion(): Promise<
    import("./index").starlightLatestVersionContext
  >;
}
