//! Implementation of the `wasm-pack build` command.

use crate::emoji;
use crate::wasm_pack::bindgen;
use crate::wasm_pack::build;
use crate::wasm_pack::cache;
use crate::wasm_pack::install;
use crate::wasm_pack::install::InstallMode;
use crate::wasm_pack::license;
use crate::wasm_pack::lockfile::Lockfile;
use crate::wasm_pack::manifest;
use crate::wasm_pack::readme;
use crate::wasm_pack::utils::{create_pkg_dir, get_crate_path};
use crate::wasm_pack::wasm_opt;
use crate::wasm_pack::PBAR;
use anyhow::{anyhow, bail, Error, Result};
use binary_install::Cache;
use clap::Args;
use log::info;
use path_clean::PathClean;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

/// Everything required to configure and run the `wasm-pack build` command.
#[allow(missing_docs)]
pub struct Build {
    pub crate_path: PathBuf,
    pub crate_data: manifest::CrateData,
    pub scope: Option<String>,
    pub disable_dts: bool,
    pub target: Target,
    pub no_pack: bool,
    pub no_opt: bool,
    pub profile: BuildProfile,
    pub mode: InstallMode,
    pub out_dir: PathBuf,
    pub out_name: Option<String>,
    pub bindgen: Option<PathBuf>,
    pub cache: Cache,
    pub extra_args: Vec<String>,
    pub extra_options: Vec<String>,
}

/// What sort of output we're going to be generating and flags we're invoking
/// `wasm-bindgen` with.
#[derive(Clone, Copy, Debug, Default)]
pub enum Target {
    /// Default output mode or `--target bundler`, indicates output will be
    /// used with a bundle in a later step.
    #[default]
    Bundler,
    /// Correspond to `--target web` where the output is natively usable as an
    /// ES module in a browser and the wasm is manually instantiated.
    Web,
    /// Correspond to `--target nodejs` where the output is natively usable as
    /// a Node.js module loaded with `require`.
    Nodejs,
    /// Correspond to `--target no-modules` where the output is natively usable
    /// in a browser but pollutes the global namespace and must be manually
    /// instantiated.
    NoModules,
    /// Correspond to `--target deno` where the output is natively usable as
    /// a Deno module loaded with `import`.
    Deno,
    /// Correspond to `--target module` where the output uses ES module syntax
    /// with source phase imports for WebAssembly modules.
    Module,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Target::Bundler => "bundler",
            Target::Web => "web",
            Target::Nodejs => "nodejs",
            Target::NoModules => "no-modules",
            Target::Deno => "deno",
            Target::Module => "module",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "bundler" | "browser" => Ok(Target::Bundler),
            "web" => Ok(Target::Web),
            "nodejs" => Ok(Target::Nodejs),
            "no-modules" => Ok(Target::NoModules),
            "deno" => Ok(Target::Deno),
            "module" => Ok(Target::Module),
            _ => bail!("Unknown target: {}", s),
        }
    }
}

/// The build profile controls whether optimizations, debug info, and assertions
/// are enabled or disabled.
#[derive(Clone, Debug)]
pub enum BuildProfile {
    /// Enable assertions and debug info. Disable optimizations.
    Dev,
    /// Enable optimizations. Disable assertions and debug info.
    Release,
    /// Enable optimizations and debug info. Disable assertions.
    Profiling,
    /// User-defined profile with --profile flag
    Custom(String),
}

/// Everything required to configure and run the `wasm-pack build` command.
#[derive(Debug, Args, Default)]
#[command(allow_hyphen_values = true, trailing_var_arg = true)]
pub struct BuildOptions {
    /// The path to the Rust crate. If not set, searches up the path from the current directory.
    #[clap()]
    pub path: Option<PathBuf>,

    /// The npm scope to use in package.json, if any.
    #[clap(long = "scope", short = 's')]
    pub scope: Option<String>,

    #[clap(long = "mode", short = 'm', default_value = "normal")]
    /// Sets steps to be run. [possible values: no-install, normal, force]
    pub mode: InstallMode,

    #[clap(long = "no-typescript")]
    /// By default a *.d.ts file is generated for the generated JS file, but
    /// this flag will disable generating this TypeScript file.
    pub disable_dts: bool,

    #[clap(long = "target", short = 't', default_value = "bundler")]
    /// Sets the target environment. [possible values: bundler, nodejs, web, no-modules, deno, module]
    pub target: Target,

    #[clap(long = "debug")]
    /// Deprecated. Renamed to `--dev`.
    pub debug: bool,

    #[clap(long = "dev")]
    /// Create a development build. Enable debug info, and disable
    /// optimizations.
    pub dev: bool,

    #[clap(long = "release")]
    /// Create a release build. Enable optimizations and disable debug info.
    pub release: bool,

    #[clap(long = "profiling")]
    /// Create a profiling build. Enable optimizations and debug info.
    pub profiling: bool,

    #[clap(long = "profile")]
    /// User-defined profile with --profile flag
    pub profile: Option<String>,

    #[clap(long = "out-dir", short = 'd', default_value = "pkg")]
    /// Sets the output directory with a relative path.
    pub out_dir: String,

    #[clap(long = "out-name")]
    /// Sets the output file names. Defaults to package name.
    pub out_name: Option<String>,

    #[clap(long = "no-pack", alias = "no-package")]
    /// Option to not generate a package.json
    pub no_pack: bool,

    #[clap(long = "no-opt", alias = "no-optimization")]
    /// Option to skip optimization with wasm-opt
    pub no_opt: bool,

    /// List of extra options to pass to `cargo build`
    pub extra_options: Vec<String>,

    #[clap(long, hide = true)]
    /// Pass-through for --no-panic-recovery
    pub no_panic_recovery: bool,

    #[deprecated(note = "runtime-detected")]
    #[allow(dead_code)]
    #[clap(long = "weak-refs", hide = true)]
    /// Enable usage of the JS weak references proposal.
    pub weak_refs: bool,

    #[deprecated(note = "automatically inferred from the Wasm features")]
    #[clap(long = "reference-types", hide = true)]
    /// Enable usage of WebAssembly reference types.
    pub reference_types: bool,
}

type BuildStep = fn(&mut Build) -> Result<()>;

impl Build {
    /// Construct a build command from the given options.
    pub fn try_from_opts(mut build_opts: BuildOptions) -> Result<Self> {
        if let Some(path) = &build_opts.path {
            if path.to_string_lossy().starts_with("--") {
                let path = build_opts.path.take().unwrap();
                build_opts
                    .extra_options
                    .insert(0, path.to_string_lossy().into_owned());
            }
        }
        let crate_path = get_crate_path(build_opts.path)?;
        let crate_data = manifest::CrateData::new(&crate_path, build_opts.out_name.clone())?;
        let out_dir = crate_path.join(PathBuf::from(build_opts.out_dir)).clean();

        let dev = build_opts.dev || build_opts.debug;
        let profile = match (
            dev,
            build_opts.release,
            build_opts.profiling,
            build_opts.profile,
        ) {
            (false, false, false, None) | (false, true, false, None) => BuildProfile::Release,
            (true, false, false, None) => BuildProfile::Dev,
            (false, false, true, None) => BuildProfile::Profiling,
            (false, false, false, Some(profile)) => BuildProfile::Custom(profile),
            // Unfortunately, `clap` doesn't expose clap's `conflicts_with`
            // functionality yet, so we have to implement it ourselves.
            _ => bail!("Can only supply one of the --dev, --release, --profiling, or --profile 'name' flags"),
        };

        Ok(Build {
            crate_path,
            crate_data,
            scope: build_opts.scope,
            disable_dts: build_opts.disable_dts,
            target: build_opts.target,
            no_pack: build_opts.no_pack,
            no_opt: build_opts.no_opt,
            profile,
            mode: build_opts.mode,
            out_dir,
            out_name: build_opts.out_name,
            bindgen: None,
            cache: cache::get_wasm_pack_cache()?,
            extra_args: Vec::new(),
            extra_options: build_opts.extra_options,
        })
    }

    /// Prepare this `Build` command.
    pub fn init(&mut self) -> Result<()> {
        let process_steps = Build::get_preprocess_steps(self.mode);
        for (_, process_step) in process_steps {
            process_step(self)?;
        }
        Ok(())
    }

    /// Execute this `Build` command.
    pub fn run(&mut self) -> Result<()> {
        let process_steps = Build::get_process_steps(self.no_pack, self.no_opt);

        let started = Instant::now();

        for (_, process_step) in process_steps {
            process_step(self)?;
        }

        let duration = crate::wasm_pack::utils::elapsed(started.elapsed());
        info!("Done in {}.", &duration);
        info!(
            "Your wasm pkg is ready to publish at {}.",
            self.out_dir.display()
        );

        PBAR.info(&format!("{}Done in {}", emoji::SPARKLE, &duration));

        PBAR.info(&format!(
            "{} Your wasm pkg is ready to publish at {}.",
            emoji::PACKAGE,
            self.out_dir.display()
        ));
        Ok(())
    }

    #[allow(clippy::vec_init_then_push)]
    fn get_preprocess_steps(mode: InstallMode) -> Vec<(&'static str, BuildStep)> {
        macro_rules! steps {
            ($($name:ident),+) => {
                {
                let mut steps: Vec<(&'static str, BuildStep)> = Vec::new();
                    $(steps.push((stringify!($name), Build::$name));)*
                        steps
                    }
                };
            ($($name:ident,)*) => (steps![$($name),*])
        }
        let mut steps = Vec::new();
        match &mode {
            InstallMode::Force => {}
            _ => {
                steps.extend(steps![
                    step_check_rustc_version,
                    step_check_crate_config,
                    step_check_for_wasm_target,
                ]);
            }
        }

        steps.extend(steps![step_install_wasm_bindgen]);
        steps
    }

    #[allow(clippy::vec_init_then_push)]
    fn get_process_steps(no_pack: bool, no_opt: bool) -> Vec<(&'static str, BuildStep)> {
        macro_rules! steps {
            ($($name:ident),+) => {
                {
                let mut steps: Vec<(&'static str, BuildStep)> = Vec::new();
                    $(steps.push((stringify!($name), Build::$name));)*
                        steps
                    }
                };
            ($($name:ident,)*) => (steps![$($name),*])
        }
        let mut steps = Vec::new();
        steps.extend(steps![
            step_build_wasm,
            step_create_dir,
            step_run_wasm_bindgen,
        ]);

        if !no_opt {
            steps.extend(steps![step_run_wasm_opt]);
        }

        if !no_pack {
            steps.extend(steps![
                step_create_json,
                step_copy_readme,
                step_copy_license,
            ]);
        }

        steps
    }

    fn step_check_rustc_version(&mut self) -> Result<()> {
        info!("Checking rustc version...");
        let version = build::check_rustc_version()?;
        let msg = format!("rustc version is {}.", version);
        info!("{}", &msg);
        Ok(())
    }

    fn step_check_crate_config(&mut self) -> Result<()> {
        info!("Checking crate configuration...");
        self.crate_data.check_crate_config()?;
        info!("Crate is correctly configured.");
        Ok(())
    }

    fn step_check_for_wasm_target(&mut self) -> Result<()> {
        info!("Checking for wasm-target...");
        build::wasm_target::check_for_wasm32_target()?;
        info!("Checking for wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self) -> Result<()> {
        info!("Building wasm...");
        build::cargo_build_wasm(&self.crate_path, self.profile.clone(), &self.extra_options)?;

        info!(
            "wasm built at {:#?}.",
            &self
                .crate_path
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
        );
        Ok(())
    }

    fn step_create_dir(&mut self) -> Result<()> {
        info!("Creating a pkg directory...");
        create_pkg_dir(&self.out_dir)?;
        info!("Created a pkg directory at {:#?}.", &self.crate_path);
        Ok(())
    }

    fn step_create_json(&mut self) -> Result<()> {
        self.crate_data.write_package_json(
            &self.out_dir,
            &self.scope,
            self.disable_dts,
            self.target,
        )?;
        info!(
            "Wrote a package.json at {:#?}.",
            &self.out_dir.join("package.json")
        );
        Ok(())
    }

    fn step_copy_readme(&mut self) -> Result<()> {
        info!("Copying readme from crate...");
        readme::copy_from_crate(&self.crate_data, &self.crate_path, &self.out_dir)?;
        info!("Copied readme from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_copy_license(&mut self) -> Result<()> {
        info!("Copying license from crate...");
        license::copy_from_crate(&self.crate_data, &self.crate_path, &self.out_dir)?;
        info!("Copied license from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self) -> Result<()> {
        info!("Identifying wasm-bindgen dependency...");
        let _lockfile = Lockfile::new(&self.crate_data)?;
        let _bindgen_version = _lockfile.require_wasm_bindgen()?;
        info!("Installing wasm-bindgen-cli...");
        use crate::binary::{GetBinary, WasmBindgen};
        let bindgen = WasmBindgen.get_binary(None)?;
        self.bindgen = Some(bindgen);
        info!("Installing wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_run_wasm_bindgen(&mut self) -> Result<()> {
        info!("Building the wasm bindings...");
        bindgen::wasm_bindgen_build(
            &self.crate_data,
            self.bindgen.as_ref().unwrap(),
            &self.out_dir,
            &self.out_name,
            self.disable_dts,
            self.target,
            self.profile.clone(),
            &self.extra_args,
            &self.extra_options,
        )?;
        info!("wasm bindings were built at {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_run_wasm_opt(&mut self) -> Result<()> {
        let mut args = match self
            .crate_data
            .configured_profile(self.profile.clone())
            .wasm_opt_args()
        {
            Some(args) => args,
            None => return Ok(()),
        };
        args.push("--all-features".into());
        // Keep the Wasm names section
        args.push("--debuginfo".into());
        info!("executing wasm-opt with {:?}", args);
        wasm_opt::run(
            &self.cache,
            &self.out_dir,
            &args,
            self.mode.install_permitted(),
        ).map_err(|e| {
            anyhow!(
                "{}\nTo disable `wasm-opt`, add `wasm-opt = false` to your package metadata in your `Cargo.toml`.", e
            )
        })
    }

    pub fn supports_target_module_and_reset_state(&self) -> Result<bool> {
        let bindgen_path = self.bindgen.as_ref().unwrap();
        let cli_version =
            semver::Version::parse(&install::get_cli_version("wasm-bindgen", bindgen_path)?)?;
        let expected_version = semver::Version::parse("0.2.102")?;
        Ok(cli_version >= expected_version)
    }
}
