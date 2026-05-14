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
	type Container,
	check,
	type Directory,
	dag,
	func,
	object,
	type Service,
} from "@dagger.io/dagger";

@object()
export class UbiHome {
	private docsContainer(source: Directory): Container {
		const docsDir = source.directory("documentation");

		return dag
			.container()
			.from("squidfunk/mkdocs-material:latest")
			.withExec([
				"pip",
				"install",
				"--upgrade",
				"mkdocs-material[imaging]==9.7.6",
				"pillow==12.1.1",
				"cairosvg==2.9.0",
				"mkdocs-awesome-nav==3.3.0",
				"mkdocs-macros-plugin==1.5.0",
				"mkdocs-git-revision-date-localized-plugin==1.5.1",
			])
			.withMountedDirectory("/docs", docsDir)
			.withMountedCache(
				"/docs/.cache",
				dag.cacheVolume("mkdocs-material-cache"),
			)
			.withWorkdir("/docs");
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
		return this.docsBuild(source).stdout();
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
		return this.docsContainer(source).withExec(["mkdocs", "build", "--strict"]);
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
		return this.docsBuild(source).directory("/docs/site");
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
			.asService({
				args: ["mkdocs", "serve", "--dev-addr", `0.0.0.0:${port}`],
			});
	}

	@func()
	rustContainer(
		@argument({
			defaultPath: ".",
			ignore: [
				"**",
				"!components",
				"!components/**",
				"!src",
				"!src/**",
				"!build.rs",
				"!Cargo.toml",
				"!Cargo.lock",
			],
		})
		source: Directory,
	): Container {
		return dag
			.container()
			.from("rust:latest")
			.withExec(["apt-get", "update"])
			.withExec([
				"apt-get",
				"install",
				"-y",
				"libdbus-1-dev",
				"pkg-config",
				"libasound2-dev",
			])
			.withExec(["rustup", "component", "add", "rustfmt"])
			.withExec(["rustup", "component", "add", "clippy"])
			.withMountedDirectory("/workspace", source)
			.withWorkdir("/workspace");
	}

	/**
	 * Check Rust code formatting with cargo fmt.
	 */
	@func()
	@check()
	async rustFmtCheck(
		@argument({
			defaultPath: ".",
			ignore: [
				"**",
				"!components",
				"!components/**",
				"!src",
				"!src/**",
				"!build.rs",
				"!Cargo.toml",
				"!Cargo.lock",
			],
		})
		source: Directory,
	): Promise<string> {
		return this.rustContainer(source)
			.withExec(["cargo", "fmt", "--all", "--", "--check"])
			.withExec([
				"cargo",
				"clippy",
				"--all-targets",
				"--all-features",
				"--",
				"-D",
				"warnings",
			])
			.stdout();
	}

	/**
	 * Check Python code style with ruff (linter and formatter check).
	 */
	@func()
	@check()
	async ruffCheck(
		@argument({
			defaultPath: ".",
			ignore: [
				"**",
				"!tests/**/*.py",
				"!tests/ruff.toml",
				"!tests/pyproject.toml",
				"!tests/uv.lock",
				"tests/.venv/**",
			],
		})
		source: Directory,
	): Promise<string> {
		return dag
			.container()
			.from("ghcr.io/astral-sh/uv:python3.14-alpine")
			.withWorkdir("/workspace/tests")
			.withFile(
				"/workspace/tests/pyproject.toml",
				source.file("tests/pyproject.toml"),
			)
			.withFile("/workspace/tests/uv.lock", source.file("tests/uv.lock"))
			.withExec(["uv", "sync", "--no-group", "e2e"])
			.withMountedDirectory("/workspace", source)
			.withExec(["uv", "run", "--no-group", "e2e", "ruff", "check"])
			.stdout();
	}

	/**
	 * Check dagger pipeline configuration with biome (formatter and linter).
	 */
	@func()
	@check()
	async biomeCheck(
		@argument({
			defaultPath: ".",

			ignore: [
				"**",
				"!*.json",
				"!components/**/*.json",
				"!.devcontainer/**/*.json",
				"!.vscode/**/*.json",
				"!.dagger",
				"!biome.json",
				"!.gitignore",
				".dagger/sdk",
			],
		})
		source: Directory,
	): Promise<string> {
		// return source;
		return dag
			.container()
			.from("node:22-slim")
			.withExec(["npm", "install", "-g", "@biomejs/biome"])
			.withMountedDirectory("/workspace", source)
			.withWorkdir("/workspace")
			.withExec(["biome", "check", "--config-path", "biome.json"])
			.stdout();
	}
}
