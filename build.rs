use std::path::PathBuf;
use std::fs;
use std::env;
use std::io::Write;
use std::process::Command;
use bindgen::builder;

fn main() {
    // Create a directory for Musashi in the target directory
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let musashi_dir = out_dir.join("musashi");
    fs::create_dir_all(&musashi_dir).unwrap();

    // Download Musashi if it doesn't exist
    if !musashi_dir.join("m68kcpu.c").exists() {
        println!("cargo:warning=Downloading Musashi...");
        let musashi_url = "https://github.com/kstenerud/Musashi/archive/refs/heads/master.zip";
        let response = reqwest::blocking::get(musashi_url).unwrap();
        let zip_file = out_dir.join("musashi.zip");
        let mut file = fs::File::create(&zip_file).unwrap();
        file.write_all(&response.bytes().unwrap()).unwrap();

        // Extract the zip file
        let mut archive = zip::ZipArchive::new(fs::File::open(&zip_file).unwrap()).unwrap();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = musashi_dir.join(file.name().replace("Musashi-master/", ""));
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
        fs::remove_file(zip_file).unwrap();
    }

    // Compile m68kmake
    println!("cargo:warning=Compiling m68kmake...");
    let status = Command::new("gcc")
        .arg("-o")
        .arg(musashi_dir.join("m68kmake"))
        .arg(musashi_dir.join("m68kmake.c"))
        .status()
        .expect("Failed to compile m68kmake");
    if !status.success() {
        panic!("Failed to compile m68kmake");
    }

    // Run m68kmake to generate the files
    println!("cargo:warning=Generating Musashi files...");
    let status = Command::new(musashi_dir.join("m68kmake"))
        .current_dir(&musashi_dir)
        .status()
        .expect("Failed to run m68kmake");
    if !status.success() {
        panic!("Failed to run m68kmake");
    }

    // Build Musashi
    cc::Build::new()
        .files([
            musashi_dir.join("softfloat/softfloat.c"),
            musashi_dir.join("m68kcpu.c"),
            musashi_dir.join("m68kops.c"),
        ])
        .include(&musashi_dir)
        .include(musashi_dir.join("softfloat"))
        .flag_if_supported("-Wno-unused-function")
        .define("OPTIMIZE_FOR_SIZE", None)
        .define("M68K_EMULATE_FPU", Some("0"))
        .compile("musashi");

    let bindings = builder()
        .header(musashi_dir.join("m68k.h").to_str().expect("Invalid path"))
        .blocklist_function("m68k_read_memory_.*")
        .blocklist_function("m68k_write_memory_.*")
        .generate()
        .expect("Unable to generate bindings");

    // bindings
    //     .write_to_file(musashi_dir.join("bindings.rs").to_str().expect("Invalid path"))
    //     .expect("Couldn't write bindings!");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=build.rs");
}