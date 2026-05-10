/**
 * A generated module for UbiHome functions
 *
 * This module has been generated via dagger init and serves as a reference to
 * basic module structure as you get started with Dagger.
 *
 * Two functions have been pre-created. You can modify, delete, or add to them,
 * as needed. They demonstrate usage of arguments and return types using simple
 * echo and grep commands. The functions can be called from the dagger CLI or
 * from one of the SDKs.
 *
 * The first line in this comment block is a short description line and the
 * rest is a long description with more detail on the module's purpose or usage,
 * if appropriate. All modules should have a short description.
 */
import {
  argument,
  dag,
  Container,
  Directory,
  Service,
  object,
  func,
  check,
} from "@dagger.io/dagger"

@object()
export class UbiHome {
  private docsContainer(source: Directory): Container {
    const docsDir = source.directory("documentation")

    return dag
      .container()
      .from("squidfunk/mkdocs-material:latest")
      .withExec(
        ["pip", "install", "--upgrade", "mkdocs-material[imaging]", "pillow", "cairosvg", "mkdocs-awesome-nav", "mkdocs-macros-plugin", "mkdocs-git-revision-date-localized-plugin"]
      )
      .withMountedDirectory("/docs", docsDir)
      .withMountedCache("/docs/.cache", dag.cacheVolume("mkdocs-material-cache"))
      .withWorkdir("/docs")
  }

  /**
   * Run strict MkDocs checks and return build output logs.
   */
  @func()
  @check()
  async docsCheck(
    @argument({
      defaultPath: ".",
      ignore: ["**", "!documentation", "!documentation/**"],
    })
    source: Directory,
  ): Promise<string> {
    return this.docsBuild(source)
      .stdout()
  }

  /**
   * Build static documentation and return the generated site directory.
   */
  @func()
  docsBuild(
    @argument({
      defaultPath: ".",
      ignore: ["**", "!documentation", "!documentation/**"],
    })
    source: Directory,
  ): Container {
    return this.docsContainer(source)
      .withExec(["mkdocs", "build", "--strict"])
  }


    /**
   * Build static documentation and return the generated site directory.
   */
  @func()
  docsBuildDir(
    @argument({
      defaultPath: ".",
      ignore: ["**", "!documentation", "!documentation/**"],
    })
    source: Directory,
  ): Directory {
    return this.docsBuild(source)
      .directory("/docs/site")
  }
  /**
   * Run a local documentation preview service.
   */
  @func()
  docsPreview(
    @argument({
      defaultPath: ".",
      ignore: ["**", "!documentation", "!documentation/**"],
    })
    source: Directory,
    port = 8000,
  ): Service {
    return this.docsContainer(source)
      .withExposedPort(port)
      .asService({args: ["mkdocs", "serve", "--dev-addr", `0.0.0.0:${port}`]})
  }
}
