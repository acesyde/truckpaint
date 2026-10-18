//! `tpv`: builds and checks TruckPaint vehicle packages.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
tpv: build and check TruckPaint vehicle packages (.tpv)

Usage:
  tpv pack <folder> [-o <file.tpv>]   Pack a folder holding a vehicle.json and its
                                      templates (PNG, SVG or DDS, converted to PNG).
                                      Default output: <id>-<version>.tpv
  tpv check <file.tpv>                Validate a package and print its contents.
  tpv extract <installation> --package-version <version>
              [--vehicle <identifier>] [--output <directory>] [--previous <batch>]
                                      Extract official ETS2 trucks and owned
                                      trailers. Output parent:
                                      dist/ by default, one unique directory per batch.
                                      --previous: the last published batch; changed
                                      templates get the next layout version.
  tpv --help                          Show this help.

Each texture's size is [width, height] in pixels, each side a power of two
from 64 to 8192: [4096, 4096], [8192, 1024]. A single number is refused.

See https://truckpaint.ace-dev.me/en/vehicles/package-format/.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("pack") => pack(&args[1..]),
        Some("check") => check(&args[1..]),
        Some("extract") => extract(&args[1..]),
        Some("-h" | "--help" | "help") => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => Err(format!("missing or unknown command\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn pack(args: &[String]) -> Result<(), String> {
    let mut folder = None;
    let mut output = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(PathBuf::from(rest.next().ok_or("-o needs a file name")?));
            }
            other if folder.is_none() && !other.starts_with('-') => {
                folder = Some(PathBuf::from(other));
            }
            other => return Err(format!("unexpected argument \"{other}\"\n\n{USAGE}")),
        }
    }
    let folder = folder.ok_or_else(|| format!("pack needs a folder\n\n{USAGE}"))?;
    let packed = tp_pack::pack_folder(&folder).map_err(|e| e.to_string())?;
    let output = output.unwrap_or_else(|| PathBuf::from(packed.file_name()));
    for (dds, png) in &packed.converted {
        println!("converted {dds} -> {png}");
    }
    for file in &packed.ignored {
        println!("ignored   {file}");
    }
    std::fs::write(&output, &packed.bytes).map_err(|e| format!("{}: {e}", output.display()))?;
    println!(
        "packed    {} ({} KB)",
        output.display(),
        packed.bytes.len().div_ceil(1024)
    );
    print!("{}", tp_pack::summary(&packed.manifest));
    Ok(())
}

fn check(args: &[String]) -> Result<(), String> {
    let [file] = args else {
        return Err(format!("check needs one file\n\n{USAGE}"));
    };
    let bytes = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
    let manifest = tp_pack::check(&bytes).map_err(|e| format!("{file}: {e}"))?;
    print!("{}", tp_pack::summary(&manifest));
    Ok(())
}

fn extract(args: &[String]) -> Result<(), String> {
    let (installation, values) = parse_options(
        args,
        &["--vehicle", "--output", "--package-version", "--previous"],
    )?;
    let installation = installation.ok_or("extract needs an ETS2 installation directory")?;
    let package_version = values
        .get("--package-version")
        .ok_or("extract needs --package-version <version>")?
        .parse::<semver::Version>()
        .map_err(|e| format!("invalid package version: {e}"))?;
    let options = tp_pack::extract::Options {
        installation: installation.into(),
        vehicle: values.get("--vehicle").map(|v| (*v).to_owned()),
        output: values.get("--output").unwrap_or(&"dist").into(),
        package_version,
        previous: values.get("--previous").map(Into::into),
    };
    let batch = tp_pack::extract::run(&options)?;
    println!("report: {}", batch.directory.join("report.json").display());
    if batch.failed == 0 {
        Ok(())
    } else {
        Err(format!(
            "incomplete batch: {} vehicle(s) failed",
            batch.failed
        ))
    }
}

/// The positional argument and the values given for `options` (`--name value`).
fn parse_options<'a>(
    args: &'a [String],
    options: &[&'a str],
) -> Result<(Option<&'a str>, HashMap<&'a str, &'a str>), String> {
    let mut positional = None;
    let mut values = HashMap::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if let Some(&name) = options.iter().find(|o| **o == arg) {
            if values.contains_key(name) {
                return Err(format!("duplicate option {arg}"));
            }
            let value = rest.next().ok_or_else(|| format!("{arg} needs a value"))?;
            if value.starts_with("--") {
                return Err(format!("{arg} needs a value"));
            }
            values.insert(name, value.as_str());
        } else if positional.is_none() && !arg.starts_with('-') {
            positional = Some(arg.as_str());
        } else {
            return Err(format!("unexpected argument \"{arg}\"\n\n{USAGE}"));
        }
    }
    Ok((positional, values))
}
