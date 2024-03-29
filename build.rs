use anyhow::Result;
use vergen::EmitBuilder;

const RSID3_VERSION_MAJOR: u8 = 0;
const RSID3_VERSION_MINOR: u8 = 1;
const RSID3_VERSION_PATCH: u8 = 0;
const RSID3_VERSION_STR: &str = "0.1.0";

pub fn main() -> Result<()> {
    // NOTE: This will output only a build timestamp and long SHA from git.
    // NOTE: This set requires the build and git features.
    // NOTE: See the EmitBuilder documentation for configuration options.
    EmitBuilder::builder()
        .build_timestamp()
        .git_sha(false)
        .emit()?;
    println!("cargo::rustc-env=RSID3_VERSION_MAJOR={}", RSID3_VERSION_MAJOR);
    println!("cargo::rustc-env=RSID3_VERSION_MINOR={}", RSID3_VERSION_MINOR);
    println!("cargo::rustc-env=RSID3_VERSION_PATCH={}", RSID3_VERSION_PATCH);
    println!("cargo::rustc-env=RSID3_VERSION_STR={}", RSID3_VERSION_STR);
    Ok(())
}
