use crate::{Error, fetch::FileInfo, log_info};
use super::{CompileInfo, BuildInfo};
use std::{
    process::Command,
    io::Write,
};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo) -> Vec<String> {
    let mut args = vec![
        src.to_string(),
        "/c".to_string(),
        "/EHsc".to_string(),
        format!("/std:{}", info.cppstd.to_ascii_lowercase()),
        format!("/Fo:{}", obj),
        // "/Gy".to_string(),
        // "/GL".to_string(),
        // "/Oi".to_string(),
    ];
    args.extend(info.incdirs.iter().map(|i| format!("/I{}", i)));
    args.extend(info.defines.iter().map(|d| format!("/D{}", d)));
    if info.config.is_release() {
        args.push("/MD".to_string());
        args.push("/O2".to_string());
    } else {
        args.push("/MDd".to_string());
        args.push("/Od".to_string());
    }
    if let Some(outfile) = info.pch {
        let cmpd = format!("{}/{}.pch", info.outdir, outfile);
        args.push(format!("/Yu{}", outfile));
        args.push(format!("/Fp{}", cmpd));
    }
    args
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo) -> Result<(), Error> {
    let mut cmd = Command::new("lib");
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.args(&info.links);
    cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l)));
    cmd.args([
        format!("/OUT:{}", info.outfile.repr),
        "/MACHINE:X64".to_string(),
        "/SUBSYSTEM:CONSOLE".to_string(),
        // "/LTCG".to_string(),
    ]);
    let output = cmd.output().unwrap();
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        println!();
        log_info!("successfully built project {}", info.outfile.repr);
        println!();
        Ok(())
    }
}

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo) -> Result<(), Error> {
    let mut cmd = Command::new("link");
    cmd.args(objs.into_iter().map(|fi| fi.repr));
    cmd.args(&info.links);
    cmd.args(DEFAULT_LIBS);
    cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l)));
    cmd.args([
        format!("/OUT:{}", info.outfile.repr),
        "/MACHINE:X64".to_string(),
        "/SUBSYSTEM:CONSOLE".to_string(),
        // "/LTCG".to_string(),
        // "/DEBUG".to_string(),
        // format!("/{}", info.config.as_arg()),
        // "/OPT:REF".to_string(),
    ]);
    let output = cmd.output().unwrap();
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        println!();
        log_info!("successfully built project {}", info.outfile.repr);
        println!();
        Ok(())
    }
}


pub mod prep {
    use crate::{fetch::FileInfo, log_info_noline};
    use super::BuildInfo;
    use std::{
        process::Command,
        path::PathBuf,
        io::Write,
    };

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
                // "/Gy".to_string(),
                // "/GL".to_string(),
                // "/Oi".to_string(),
            ]);
            cmd.args(info.incdirs.iter().map(|i| format!("/I{}", i)));
            cmd.args(info.defines.iter().map(|d| format!("/D{}", d)));
            if info.config.is_release() {
                cmd.args(["/MD", "/O2"]);
            } else {
                cmd.args(["/MDd", "/Od"]);
            }
            log_info_noline!("compiling precompiled header: ");
            let _ = std::io::stdout().flush();
            std::io::stdout().write_all(&cmd.output().unwrap().stdout).unwrap();
            println!();
        }
    }
}


const DEFAULT_LIBS: &[&str] = &[
    "kernel32.lib",
    "user32.lib",
    "winspool.lib",
    "comdlg32.lib",
    "advapi32.lib",
    "shell32.lib",
    "ole32.lib",
    "oleaut32.lib",
    "uuid.lib",
    "odbc32.lib",
    "odbccp32.lib",
];

