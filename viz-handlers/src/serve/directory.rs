//! Directory
//!
//! <https://github.com/vercel/serve-handler> MIT

use std::{
    ffi::OsStr,
    fmt::{Display, Formatter, Result},
    fs::read_dir,
    path::{Path, PathBuf},
    str::FromStr,
    string::ToString,
};

use viz_core::{IntoResponse, Response, ResponseExt};

#[derive(Debug)]
pub(crate) struct Directory {
    name: String,
    paths: Paths,
    files: Files,
}

impl Directory {
    pub(crate) fn new(
        base: &str,
        prev: bool,
        root: &Path,
        unlisted: &Option<Vec<&'static str>>,
    ) -> Option<Directory> {
        let mut entries = read_dir(root).ok()?;

        let mut files = Vec::new();

        while let Some(Ok(entry)) = entries.next() {
            let name = entry.file_name().to_str()?.to_string();

            if unlisted
                .as_ref()
                .filter(|unlisted| unlisted.contains(&name.as_str()))
                .is_some()
            {
                continue;
            }

            let kind = entry.file_type().ok()?.is_file();
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(OsStr::to_str)
                .map(ToString::to_string);

            let mut url = base.trim_matches('/').to_owned();
            url.push('/');
            url.push_str(path.strip_prefix(root).ok()?.to_str()?);

            files.push((url, name.clone(), kind, ext, name));
        }

        files.sort_by_key(|f| f.1.clone());

        let curr = PathBuf::from_str(base).ok()?;

        if prev {
            let parent = curr.parent()?;
            files.insert(
                0,
                (
                    parent.join("").to_str()?.strip_prefix('/')?.to_string(),
                    parent
                        .file_name()
                        .and_then(OsStr::to_str)
                        .unwrap_or("")
                        .to_string(),
                    false,
                    None,
                    "..".to_string(),
                ),
            );
        }

        let mut paths = Vec::new();

        for a in curr.ancestors() {
            if let (Some(u), Some(n)) = (a.join("").to_str(), a.file_name().and_then(OsStr::to_str))
            {
                paths.push((u.to_string(), n.to_owned() + "/"));
            }
        }

        paths.reverse();

        Some(Directory {
            name: base.to_string(),
            paths: Paths(paths),
            files: Files(files),
        })
    }
}

impl IntoResponse for Directory {
    fn into_response(self) -> Response {
        Response::html(self.to_string())
    }
}

impl Display for Directory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            include_str!("list.tpl"),
            name = &self.name,
            paths = &self.paths,
            files = &self.files
        )
    }
}

/// Path: (url, name)
#[derive(Debug)]
pub(crate) struct Paths(Vec<(String, String)>);

impl Display for Paths {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (url, name) in &self.0 {
            writeln!(f, r#"<a href="{}">{}</a>"#, &url, &name)?;
        }
        Ok(())
    }
}

/// File: (relative, title, kind, ext, base)
#[derive(Debug)]
pub(crate) struct Files(Vec<(String, String, bool, Option<String>, String)>);

impl Display for Files {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (relative, title, kind, ext, base) in &self.0 {
            writeln!(
                f,
                r#"<li><a href="/{}" title="{}" class="{} {}">{}</a></li>"#,
                &relative,
                &title,
                if *kind { "file" } else { "folder" },
                match &ext {
                    Some(ext) => ext,
                    None => "",
                },
                &base
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serve_directory() {
        let d = Directory {
            name: "viz".to_string(),
            paths: Paths(vec![("src/main.rs".to_string(), "main".to_string())]),
            files: Files(vec![
                (
                    ".".to_string(),
                    "src".to_string(),
                    true,
                    None,
                    "src".to_string(),
                ),
                (
                    ".".to_string(),
                    "lib".to_string(),
                    false,
                    Some("rs".to_string()),
                    "lib.rs".to_string(),
                ),
            ]),
        };
        assert!(!d.to_string().is_empty());
    }
}
