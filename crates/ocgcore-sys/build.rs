use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let c_core = root.join("c-core");
    let lua_dir = c_core.join("lua");
    let lua_src = lua_dir.join("src");

    println!("cargo:rerun-if-changed=c-core");

    // 1. Compile Lua 5.4 as C++ (matching EDOPro/ygopro requirements for C++ exception handling)
    let lua_files = [
        "lapi.c",
        "lauxlib.c",
        "lbaselib.c",
        "lcode.c",
        "lcorolib.c",
        "lctype.c",
        "ldblib.c",
        "ldebug.c",
        "ldo.c",
        "ldump.c",
        "lfunc.c",
        "lgc.c",
        "linit.c",
        "liolib.c",
        "llex.c",
        "lmathlib.c",
        "lmem.c",
        "loadlib.c",
        "lobject.c",
        "lopcodes.c",
        "loslib.c",
        "lparser.c",
        "lstate.c",
        "lstring.c",
        "lstrlib.c",
        "ltable.c",
        "ltablib.c",
        "ltm.c",
        "lundump.c",
        "lutf8lib.c",
        "lvm.c",
        "lzio.c",
    ];

    let mut lua_build = cc::Build::new();
    lua_build
        .cpp(true)
        .std("c++17")
        .include(&lua_src)
        .include(&lua_dir)
        .flag("-x")
        .flag("c++")
        .flag("-include")
        .flag(lua_dir.join("luaconf-customize.h").to_str().unwrap());

    for f in &lua_files {
        lua_build.file(lua_src.join(f));
    }
    lua_build.compile("lua");

    // 2. Compile ocgcore
    let ocgcore_files = [
        "card.cpp",
        "duel.cpp",
        "effect.cpp",
        "field.cpp",
        "interpreter.cpp",
        "libcard.cpp",
        "libdebug.cpp",
        "libduel.cpp",
        "libeffect.cpp",
        "libgroup.cpp",
        "ocgapi.cpp",
        "operations.cpp",
        "playerop.cpp",
        "processor.cpp",
        "processor_visit.cpp",
        "scriptlib.cpp",
    ];

    let mut ocg_build = cc::Build::new();
    ocg_build
        .cpp(true)
        .std("c++17")
        .include(&c_core)
        .include(&lua_src)
        .include(&lua_dir);

    // Suppress minor warnings from upstream C++ codebase
    ocg_build.flag("-Wno-unused-parameter");
    ocg_build.flag("-Wno-sign-compare");
    ocg_build.flag("-Wno-missing-field-initializers");

    for f in &ocgcore_files {
        ocg_build.file(c_core.join(f));
    }
    ocg_build.compile("ocgcore");

    // Link C++ standard library
    println!("cargo:rustc-link-lib=stdc++");
}
