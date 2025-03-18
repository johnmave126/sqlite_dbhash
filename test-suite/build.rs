use std::{
    env,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

fn main() {
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("cannot find environment variable 'OUT_DIR'"));
    let mut cfg = cc::Build::new();
    let sqlite3_include = std::env::var_os("DEP_SQLITE3_INCLUDE")
        .expect("cannot find environment variable 'DEP_SQLITE3_INCLUDE'");
    cfg.include(sqlite3_include);
    let sqlite3_lib = std::env::var_os("DEP_SQLITE3_LIB_DIR")
        .expect("cannot find environment variable 'DEP_SQLITE3_LIB_DIR'");

    let cc = cfg.get_compiler();
    let mut cmd = cc.to_command();

    cmd.arg("sqlite/tool/dbhash.c");
    if cc.is_like_msvc() {
        let mut libpath_arg = OsString::from("-LIBPATH:");
        libpath_arg.push(sqlite3_lib);
        cmd.args([
            OsStr::new("-Fo:"),
            &out_dir.join("").into_os_string(),
            OsStr::new("-Fe:"),
            &out_dir.join("dbhash.exe").into_os_string(),
            OsStr::new("-link"),
            OsStr::new("sqlite3.lib"),
            &libpath_arg,
        ]);
    } else {
        cmd.args([
            OsStr::new("-o"),
            &out_dir.join("dbhash").into_os_string(),
            OsStr::new("-L"),
            &sqlite3_lib,
            OsStr::new("-lsqlite3"),
            OsStr::new("-lm"),
        ]);
    };

    assert!(cmd.status().expect("failed to compile dbhash").success());

    println!(
        "cargo:rustc-env=DBHASH_PATH={}",
        out_dir.join("dbhash").display()
    );
    println!("cargo:rerun-if-changed=sqlite/tool/dbhash.c");
}
