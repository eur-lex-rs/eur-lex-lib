use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eur_lex_lib::loader::load_act;
use eur_lex_lib::model::Act;

/// Print a Formex act in the chosen format.
///
/// Pass a local directory path, or use `--celex` to fetch directly from the
/// EUR-Lex Cellar repository. The directory must contain a `*.doc.fmx.xml`
/// or `*.doc.xml` registry file.
#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct Cli {
    /// Path to the Formex act directory (conflicts with --celex).
    dir: Option<PathBuf>,

    /// Fetch an act from EUR-Lex Cellar by CELEX number (e.g. 32022R2065).
    #[arg(short, long, conflicts_with = "dir")]
    celex: Option<String>,

    /// Output format.
    #[arg(short, long, default_value = "md")]
    format: Format,
}

#[derive(Clone, ValueEnum)]
enum Format {
    /// Markdown (GitHub-flavoured).
    Md,
    /// Plain text.
    Txt,
    /// Self-contained HTML5 document.
    Html,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let act = match (cli.celex.as_deref(), cli.dir.as_deref()) {
        (Some(celex), _) => fetch_by_celex(celex)?,
        (None, Some(dir)) => load_act(dir)?,
        (None, None) => unreachable!("clap enforces arg_required_else_help"),
    };

    let output = match cli.format {
        Format::Md => eur_lex_lib::fmt::markdown::render_act(&act),
        Format::Txt => eur_lex_lib::fmt::txt::render_act(&act),
        Format::Html => eur_lex_lib::fmt::html::render_act(&act),
    };
    print!("{output}");
    Ok(())
}

fn fetch_by_celex(celex: &str) -> Result<Act, Box<dyn std::error::Error>> {
    let url = format!("http://publications.europa.eu/resource/celex/{celex}");
    let bytes = reqwest::blocking::Client::new()
        .get(&url)
        .header("Accept", "application/zip;mtype=fmx4")
        .header("Accept-Language", "eng")
        .send()?
        .error_for_status()?
        .bytes()?;

    let tmp = tempfile::tempdir()?;
    zip::ZipArchive::new(std::io::Cursor::new(bytes))?.extract(tmp.path())?;

    // `tmp` must remain in scope until load_act returns.
    Ok(load_act(tmp.path())?)
}
