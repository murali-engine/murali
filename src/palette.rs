use std::path::{Path, PathBuf};

use glam::Vec4;
use indexmap::IndexMap;

use crate::colors::from_hex;
use crate::utils::project::find_project_root;

/// An ordered, named map of colours.
///
/// Keys preserve insertion order, so iterating or calling `.cycle(i)` produces
/// colours in the same sequence they appear in the TOML file or builder calls.
#[derive(Debug, Clone)]
pub struct Palette {
    /// Human-readable palette name (defaults to the filename stem).
    pub name: String,
    colors: IndexMap<String, Vec4>,
}

impl Palette {
    /// Look up a colour by name. Returns `None` if the key is not in the palette.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Vec4> {
        self.colors.get(key).copied()
    }

    /// Look up a colour by name.
    ///
    /// # Panics
    /// Panics with a clear message if the key is missing. Use this in scene
    /// setup code where a missing name is always a programming error.
    pub fn require(&self, key: &str) -> Vec4 {
        self.colors.get(key).copied().unwrap_or_else(|| {
            panic!(
                "[murali] Palette '{}' has no colour named '{key}'. Available: [{}]",
                self.name,
                self.colors.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })
    }

    /// Iterate over `(name, colour)` pairs in insertion order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, Vec4)> + '_ {
        self.colors.iter().map(|(k, v)| (k.as_str(), *v))
    }

    /// Number of colours in this palette.
    #[must_use]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns `true` if no colours are defined.
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Return the colour at position `index % len`, wrapping around.
    ///
    /// Returns `None` if the palette is empty.
    ///
    /// Use [`Self::cycle_or_panic`] if you want a guaranteed colour and prefer
    /// a panic on empty palettes (recommended in scene setup code).
    #[must_use]
    pub fn cycle(&self, index: usize) -> Option<Vec4> {
        if self.colors.is_empty() {
            return None;
        }
        let idx = index % self.colors.len();
        self.colors.get_index(idx).map(|(_, v)| *v)
    }

    /// Return the colour at position `index % len`, wrapping around.
    ///
    /// # Panics
    /// Panics if the palette is empty.
    #[must_use]
    pub fn cycle_or_panic(&self, index: usize) -> Vec4 {
        self.cycle(index).unwrap_or_else(|| {
            panic!(
                "[murali] Palette '{}' is empty, so cycle({index}) cannot return a colour.",
                self.name
            )
        })
    }

    /// Load a palette by name, searching in order:
    ///
    /// 1. `{project_root}/palettes/{name}.toml`
    /// 2. Empty palette (never panics — missing palettes are not fatal).
    ///
    /// Use [`Palette::inline`] to load from the `[palette]` section of
    /// `murali.toml`.
    pub fn load(name: &str) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = find_project_root(&cwd);

        let path = project_root.join("palettes").join(format!("{name}.toml"));

        match Self::from_path(&path) {
            Ok(mut p) => {
                if p.name.is_empty() {
                    p.name = name.to_string();
                }
                p
            }
            Err(e) => {
                if path.exists() {
                    eprintln!("[murali] Warning: could not load palette '{name}': {e}");
                }
                Palette::empty(name)
            }
        }
    }

    /// Load the inline palette from the `[palette]` table in `murali.toml`.
    ///
    /// Returns an empty palette if `murali.toml` doesn't exist or has no
    /// `[palette]` section.
    pub fn inline() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = find_project_root(&cwd);
        let toml_path = project_root.join("murali.toml");

        if !toml_path.exists() {
            return Palette::empty("inline");
        }

        let contents = match std::fs::read_to_string(&toml_path) {
            Ok(s) => s,
            Err(_) => return Palette::empty("inline"),
        };

        let table = match contents.parse::<toml::Table>() {
            Ok(t) => t,
            Err(_) => return Palette::empty("inline"),
        };

        let Some(palette_table) = table.get("palette").and_then(|v| v.as_table()) else {
            return Palette::empty("inline");
        };

        let mut colors = IndexMap::new();
        for (k, v) in palette_table {
            match v.as_str() {
                Some(hex_str) => match from_hex(hex_str) {
                    Ok(color) => {
                        colors.insert(k.clone(), color);
                    }
                    Err(e) => {
                        eprintln!(
                            "[murali] Warning: invalid colour '{k} = {hex_str}' in [palette]: {e}"
                        );
                    }
                },
                None => {
                    eprintln!(
                        "[murali] Warning: [palette] entry '{k}' must be a hex string; ignoring non-string value."
                    );
                }
            }
        }

        Palette {
            name: "inline".to_string(),
            colors,
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read '{}': {e}", path.display()))?;
        Self::from_toml_str(&contents)
    }

    pub fn from_toml_str(s: &str) -> Result<Self, String> {
        let table = s
            .parse::<toml::Table>()
            .map_err(|e| format!("TOML parse error: {e}"))?;

        let name = table
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut colors = IndexMap::new();
        for (k, v) in &table {
            if k == "name" {
                continue;
            }
            match v.as_str() {
                Some(hex_str) => {
                    let color = from_hex(hex_str)
                        .map_err(|e| format!("invalid colour '{k} = {hex_str}': {e}"))?;
                    colors.insert(k.clone(), color);
                }
                None => {
                    return Err(format!(
                        "palette entry '{k}' must be a hex string, got a non-string value"
                    ));
                }
            }
        }

        Ok(Palette { name, colors })
    }

    pub fn builder(name: impl Into<String>) -> PaletteBuilder {
        PaletteBuilder {
            name: name.into(),
            colors: IndexMap::new(),
        }
    }

    pub fn empty(name: impl Into<String>) -> Self {
        Palette {
            name: name.into(),
            colors: IndexMap::new(),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.colors.keys().map(|k| k.as_str())
    }
}

pub struct PaletteBuilder {
    name: String,
    colors: IndexMap<String, Vec4>,
}

impl PaletteBuilder {
    pub fn add(mut self, key: impl Into<String>, color: Vec4) -> Self {
        self.colors.insert(key.into(), color);
        self
    }

    pub fn add_hex(mut self, key: impl Into<String>, hex: &str) -> Self {
        let key = key.into();
        match from_hex(hex) {
            Ok(color) => {
                self.colors.insert(key, color);
            }
            Err(e) => {
                eprintln!("[murali] PaletteBuilder: invalid hex for '{key}': {e}");
            }
        }
        self
    }

    pub fn build(self) -> Palette {
        Palette {
            name: self.name,
            colors: self.colors,
        }
    }
}
