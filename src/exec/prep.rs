use std::{io::Write, path::{Path, PathBuf}, process::Command};
use crate::{fetch::FileInfo, log_info_noline};
use super::BuildInfo;


pub fn assert_out_dirs(sdir: &str, odir: &str) {
    if !std::fs::exists("./bin/").unwrap() {
        std::fs::create_dir("./bin/").unwrap();
        std::fs::create_dir("./bin/debug/").unwrap();
        std::fs::create_dir("./bin/release/").unwrap();
    } else {
        if !std::fs::exists("./bin/debug").unwrap() {
            std::fs::create_dir("./bin/debug/").unwrap();
        }
        if !std::fs::exists("./bin/release").unwrap() {
            std::fs::create_dir("./bin/release/").unwrap();
        }
    }
    assert_out_dirs_rec(&PathBuf::from(sdir), sdir, odir);
}

pub fn assert_out_dirs_rec(root: &Path, sdir: &str, odir: &str) {
    let obj = root.to_string_lossy().replace(sdir, odir);
    if !std::fs::exists(&obj).unwrap() {
        std::fs::create_dir(obj).unwrap();
    }
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_dir() {
            assert_out_dirs_rec(&e.path(), sdir, odir);
        }
    }
}

pub fn precompile_header(header: &str, info: &BuildInfo) {
    let head_with_dir = format!("{}{}", info.srcdir, header);
    let cppf = format!("{}{}", info.srcdir, header.replace(".h", ".cpp"));
    let objt = format!("{}{}", info.outdir, header.replace(".h", ".obj"));
    let cmpd = format!("{}{}.pch", info.outdir, header.replace(&info.srcdir, &info.outdir));
    let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
    let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

    if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
        let mut cmd = Command::new("cl");
        cmd.args([
            cppf.clone(),
            "/c".to_string(),
            "/EHsc".to_string(),
            format!("/Yc{}", header),
            format!("/Fp{}", cmpd),
            format!("/std:{}", info.cppstd),
            format!("/Fo:{}", objt),
//            "/Gy".to_string(),
//            "/GL".to_string(),
//            "/Oi".to_string(),
        ]);
        cmd.args(info.incdirs.iter().map(|i| format!("/I{}", i)));
        cmd.args(info.defines.iter().map(|d| format!("/D{}", d)));
        if info.config.is_release() {
            cmd.args(["/MD", "/O2"]);
        } else {
            cmd.args(["/MDd", "/Od"]);
        }
        log_info_noline!("compiling precompiled header: ");
        std::io::stdout().write_all(&cmd.output().unwrap().stdout).unwrap();
        println!();
    }
}

pub fn precompile_header_gcc(header: &str, info: &BuildInfo) {
    let head_with_dir = format!("{}{}", info.srcdir, header);
    let cmpd = format!("{}{}.gch", info.outdir, header.replace(&info.srcdir, &info.outdir));
    let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
    let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

    if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
        let mut cmd = Command::new("g++");
        cmd.args([
            "-c".to_string(),
            format!("-Yc{}", header),
            format!("-Fp{}", cmpd),
            format!("-std:{}", info.cppstd),
        ]);
        cmd.args(info.incdirs.iter().map(|i| format!("-I{}", i)));
        cmd.args(info.defines.iter().map(|d| format!("-D{}", d)));
        if info.config.is_release() {
            cmd.args(["/O2"]);
        }
        log_info_noline!("compiling precompiled header: ");
        std::io::stdout().write_all(&cmd.output().unwrap().stdout).unwrap();
        println!();
    }
}
