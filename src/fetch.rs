use std::{
    path::{Path, PathBuf},
    io::Write,
};
use crate::{
    BuildFile, LibFile, ProjKind, Config,
    error::Error,
    log_info,
};


#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub path: PathBuf,
    pub repr: String,
    pub exists: bool,
    pub modified: Option<std::time::SystemTime>
}

#[allow(dead_code)]
impl FileInfo {
    pub fn from_path(path: &Path) -> Self {
        let exists = path.exists();
        let modified = if exists {
            Some(std::fs::metadata(path).unwrap().modified().unwrap())
        } else {
            None
        };
        let path = path.to_owned();
        let repr = path.to_string_lossy().to_string();

        Self{
            path,
            repr,
            exists,
            modified,
        }
    }

    pub fn from_str(path: &str) -> Self {
        Self::from_path(&PathBuf::from(path))
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
    pub fn exists(&self) -> bool {
        self.exists
    }
    pub fn modified(&self) -> Option<std::time::SystemTime> {
        self.modified
    }
}


pub fn get_source_files(sdir: &Path, ext: &str) -> Option<Vec<FileInfo>> {
    let mut res = Vec::new();

    for e in std::fs::read_dir(sdir).ok()? {
        let e = e.ok()?;
        if e.path().is_dir() {
            res.extend(get_source_files(&e.path(), ext)?);
        } else {
            let filename = e.path().file_name()?.to_str()?.to_string();
            if filename.ends_with(ext) && filename != "pch.cpp" {
                res.push(FileInfo::from_path(&e.path()));
            }
        }
    }

    Some(res)
}

pub fn get_project_kind(sdir: &str) -> Result<ProjKind, Error> {
    for e in std::fs::read_dir(sdir).map_err(|_| Error::MissingEntryPoint)? {
        let e = e.map_err(|_| Error::MissingEntryPoint)?;
        if e.path().is_file() {
            let filename = e.path().file_name().unwrap().to_str().unwrap().to_string();
            if filename.ends_with("main.cpp") {
                return Ok(ProjKind::App)
            }
            if filename.ends_with("main.c") {
                return Ok(ProjKind::App)
            }
            if filename.ends_with("lib.hpp") {
                return Ok(ProjKind::Lib)
            }
            if filename.ends_with("lib.h") {
                return Ok(ProjKind::Lib)
            }
        }
    }
    Err(Error::MissingEntryPoint)
}


#[derive(Debug, Clone)]
pub struct Dependencies {
    pub incdirs: Vec<String>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub defines: Vec<String>,
}


pub fn get_libraries(libraries: Vec<String>, config: Config, maxcpp: &str) -> Result<Dependencies, Error> {
    let mut incdirs = Vec::new();
    let mut libdirs = Vec::new();
    let mut links = Vec::new();
    let mut defines = Vec::new();

    for lib in libraries {
        let (name, version) = get_version(&lib);

        if let Ok(build) = std::fs::read_to_string(format!("lib/{}/lib.json", name)) {
            let libinfo = LibFile::from_str(&build)?
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("lib/{}/{}", name, libinfo.incdir));
            libdirs.push(format!("lib/{}/{}", name, libinfo.libdir));
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
        } else if let Ok(build) = std::fs::read_to_string(format!("{}/lib.json", name)) {
            let libinfo = LibFile::from_str(&build)?
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("{}/{}", name, libinfo.incdir));
            libdirs.push(format!("{}/{}", name, libinfo.libdir));
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
        } else if let Ok(build) = std::fs::read_to_string(format!("{}/build.json", name)) {
            let build: BuildFile = serde_json::from_str(&build).unwrap();
            log_info!("building project dependency: {}", build.project);
            let save = std::env::current_dir().unwrap();
            std::env::set_current_dir(name).unwrap();
            std::process::Command::new("mscmp")
                .arg("build")
                .arg(format!("-{}", config))
                .status()
                .unwrap();
            std::env::set_current_dir(&save).unwrap();
            let libinfo = LibFile::from(build)
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("{}/{}", name, libinfo.incdir));
            libdirs.push(format!("{}/{}", name, libinfo.libdir));
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
            println!();
        }
    }

    Ok(Dependencies{
        incdirs,
        libdirs,
        links,
        defines,
    })
}


fn get_version(s: &str) -> (&str, Option<&str>) {
    for (i, c) in s.chars().rev().enumerate() {
        if c == '/' || c == '\\' {
            return (s, None)
        } else if c == '.' {
            let l = s.len();
            return (&s[..(l-i-1)], Some(&s[(l-i)..]))
        }
    }
    (s, None)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_version() {
        assert_eq!(get_version("SFML"),             ("SFML", None));
        assert_eq!(get_version("SFML.static"),      ("SFML", Some("static")));
        assert_eq!(get_version("SF.ML.static"),     ("SF.ML", Some("static")));
        assert_eq!(get_version("../Rusty"),         ("../Rusty", None));
        assert_eq!(get_version("../Rusty.static"),  ("../Rusty", Some("static")));
        assert_eq!(get_version("../Ru.sty.static"), ("../Ru.sty", Some("static")));
    }
}

