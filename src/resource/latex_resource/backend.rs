use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::resource::latex_resource::error::LatexError; // Updated paths
use crate::resource::latex_resource::template::latex_document;

pub struct LatexResource {
    pub svg_content: String,
    pub hash: String,
    pub svg_path: PathBuf,
}

/// Compiles LaTeX source to SVG.
/// Now designed to be called by the Resource Cache, not the Frontend.
pub fn compile_latex(latex_src: &str, cache_dir: &Path) -> Result<LatexResource, LatexError> {
    // 1. Generate a unique hash for this source to avoid redundant work
    let mut hasher = Sha256::new();
    hasher.update("murali-latex-v2-displaystyle");
    hasher.update(latex_src);
    let hash = hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
            output
        });

    let work_dir = cache_dir.join(&hash);
    fs::create_dir_all(&work_dir)?;

    let tex_path = work_dir.join("expr.tex");
    let dvi_path = work_dir.join("expr.dvi");
    let svg_path = work_dir.join("expr.svg");

    // 2. Check if we already compiled this in a previous run
    if svg_path.exists() {
        let svg_content = fs::read_to_string(&svg_path)?;
        return Ok(LatexResource {
            svg_content,
            hash,
            svg_path,
        });
    }

    // 3. Write .tex using the template
    fs::write(&tex_path, latex_document(latex_src))?;

    // 4. Run latex → dvi
    // We use -output-directory to keep the cache clean
    let latex_out = run_latex(&tex_path, &work_dir).map_err(|_| LatexError::LatexNotFound)?;

    if !latex_out.status.success() {
        return Err(LatexError::LatexFailed(
            String::from_utf8_lossy(&latex_out.stdout).to_string(),
        ));
    }

    // 5. Run dvisvgm (using --no-fonts to convert glyphs to SVG paths)
    // This makes the output backend-agnostic as we don't need to load .ttf files
    let mut dvisvgm_cmd = Command::new("dvisvgm");
    dvisvgm_cmd
        .arg("--no-fonts")
        .arg("--precision=3")
        .arg("-o")
        .arg(&svg_path)
        .arg(&dvi_path);
    let dvisvgm_out = run_dvisvgm(dvisvgm_cmd).map_err(|_| LatexError::DviSvgmNotFound)?;

    if !dvisvgm_out.status.success() {
        return Err(LatexError::DviSvgmFailed(
            String::from_utf8_lossy(&dvisvgm_out.stderr).to_string(),
        ));
    }

    let svg_content = fs::read_to_string(&svg_path)?;

    Ok(LatexResource {
        svg_content,
        hash,
        svg_path,
    })
}

#[derive(Debug, Default, Clone)]
struct TexEnv {
    texmfcnf_dir: Option<PathBuf>,
}

fn detect_tex_env() -> TexEnv {
    let texmfcnf_dir = Command::new("kpsewhich")
        .arg("texmf.cnf")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                PathBuf::from(path).parent().map(|p| p.to_path_buf())
            }
        });

    TexEnv { texmfcnf_dir }
}

fn apply_tex_env(cmd: &mut Command, env: &TexEnv) {
    if let Some(texmfcnf_dir) = &env.texmfcnf_dir {
        cmd.env("TEXMFCNF", texmfcnf_dir);
    }
}

fn run_latex(tex_path: &Path, work_dir: &Path) -> std::io::Result<std::process::Output> {
    let mut latex_cmd = Command::new("latex");
    latex_cmd
        .arg("-interaction=nonstopmode")
        .arg("-output-directory")
        .arg(work_dir)
        .arg(tex_path);
    latex_cmd.output()
}

fn run_dvisvgm(mut cmd: Command) -> std::io::Result<std::process::Output> {
    let plain = cmd.output()?;
    if plain.status.success() {
        return Ok(plain);
    }

    let stderr = String::from_utf8_lossy(&plain.stderr);
    let likely_tex_config_issue = stderr.contains("tex.pro")
        || stderr.contains("TeXDict")
        || stderr.contains("default map files");
    if !likely_tex_config_issue {
        return Ok(plain);
    }

    let tex_env = detect_tex_env();
    let mut retry_cmd = Command::new("dvisvgm");
    retry_cmd.args(cmd.get_args());
    apply_tex_env(&mut retry_cmd, &tex_env);
    retry_cmd.output()
}
