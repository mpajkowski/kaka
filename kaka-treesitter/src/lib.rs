use std::{mem, path::PathBuf, process::Command};

use anyhow::{ensure, Result};
use kaka_core::languages::Languages;
use libloading_mini::Library;
use tree_sitter::{Language, Parser};

pub use tree_sitter;

pub trait LanguageLoader {
    fn load_parser(&self, lang: &str) -> Result<Option<Parser>>;
}

impl LanguageLoader for Languages {
    fn load_parser(&self, lang: &str) -> Result<Option<Parser>> {
        let repo = match self.languages.get(lang) {
            Some(lang) => &lang.treesitter,
            None => return Ok(None),
        };

        log::info!("Loading language {lang}");

        let lang = match load_lang(repo)? {
            Some(lang) => lang,
            None => return Ok(None),
        };

        let mut parser = Parser::new();
        parser.set_language(lang)?;

        Ok(Some(parser))
    }
}

fn load_lang(repo: &str) -> Result<Option<Language>> {
    let root = env!("CARGO_MANIFEST_DIR").parse::<PathBuf>().unwrap();

    let langpath = root.join("languages");
    let mut dlpath = langpath.join("obj").join(repo);
    dlpath.set_extension("so");

    if !dlpath.exists() {
        log::info!("Compiling {repo}");
        let src_path = langpath.join("src").join(repo).join("src");
        if !src_path.exists() {
            return Ok(None);
        }

        build_lang(src_path, dlpath.clone())?;
    }

    let library = Library::new(dlpath).unwrap();
    let sym = library.get(repo.replace('-', "_").as_bytes()).unwrap();

    let language = unsafe {
        let fun = mem::transmute::<_, unsafe extern "C" fn() -> Language>(sym);
        fun()
    };

    mem::forget(library);

    Ok(Some(language))
}

fn build_lang(src_path: PathBuf, dlpath: PathBuf) -> Result<()> {
    let parser_path = src_path.join("parser.c");
    let scanner_path = src_path.join("scanner.c");

    let mut config = cc::Build::new();
    config
        .cargo_metadata(false)
        .opt_level(3)
        .target(env!("TARGET"))
        .host(env!("HOST"));

    let compiler = config.get_compiler();

    let mut command = Command::new(compiler.path());

    for (env_key, env_value) in compiler.env() {
        command.env(env_key, env_value);
    }

    command.args(compiler.args());

    command
        .arg("-shared")
        .arg("-fPIC")
        .arg("-fno-exceptions")
        .arg("-g")
        .arg("-I")
        .arg(src_path)
        .arg("-o")
        .arg(&dlpath)
        .arg("-O3")
        .arg("-xc")
        .arg("-std=c11")
        .arg(scanner_path)
        .arg(parser_path);

    let output = command.output()?;
    let status = output.status;

    log::info!("Compilation status: {status}");

    ensure!(
        status.success(),
        "Failed to build {}\nstdout:\n{}\nstderr:\n{}",
        dlpath.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}
