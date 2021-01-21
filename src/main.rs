#![feature(bool_to_option)]
extern crate ferris_print;
use cargo_metadata::MetadataCommand;
use serde::Serialize;
use serde_json::to_string;
use std::{
	fs::{File, copy, create_dir},
	io::{Read, Write},
	path::Path,
	process::Command
};
use tempdir::TempDir;
use walkdir::WalkDir;
use zip::{ZipWriter, result::ZipResult, write::FileOptions};

#[derive(Serialize)]
struct NPMPackage<'a> {
	name: &'a str,
	bin: &'a str
}

fn main() {
	let metadata = MetadataCommand::new().exec().unwrap();
	let package = &metadata.resolve.unwrap().root.unwrap();
	let package = &metadata.packages.iter().find(|this| package == &this.id)
		.unwrap();
	let name = &package.name;
	let package = &package.targets[0].name;
	let target = metadata.target_directory;

	ferrisprint!("Packaging for NPM.");

	const TARGETS: [&'static str; 2] = [
		"x86_64-unknown-linux-gnu",
		"x86_64-pc-windows-gnu"
	];

	TARGETS.iter().for_each(|target| {
		let status = Command::new("cargo")
			.args(&["build", &format!("--target={}", target), "--release"])
			.status().unwrap();
		status.success().then_some(()).unwrap();
	});

	ferrisprint!("Hot diggity, it compiled.\nTime to through it into an npm package.");
	let tmp = TempDir::new("delivery").unwrap();
	let tmp_path = tmp.path().to_owned();

	create_dir(tmp_path.join("bin")).unwrap();
	copy(target.join("x86_64-pc-windows-gnu").join("release").join(format!("{}.exe", package)),
		tmp_path.join("bin").join("x86_64-windows.exe")).unwrap();
	copy(target.join("x86_64-unknown-linux-gnu").join("release").join(package),
		tmp_path.join("bin").join("x86_64-linux")).unwrap();

	let mut index = File::create(tmp_path.join("index.js")).unwrap();
	index.write_all(include_bytes!("../assets/index.js")).unwrap();
	index.sync_all().unwrap();

	let mut metadata = File::create(tmp_path.join("package.json")).unwrap();
	metadata.write_all(to_string(&NPMPackage {
		name: name,
		bin: "index.js"
	}).unwrap().as_bytes()).unwrap();

	zip_dir(&target.join("package").join("npm.zip"), &tmp_path).unwrap();
}

fn zip_dir(zip_file: &Path, src_dir: &Path) -> ZipResult<()> {
	eprintln!("writing contents of {:?} to {:?}", src_dir, zip_file);
	let mut zip = ZipWriter::new(File::create(zip_file).unwrap());
	let walker = WalkDir::new(src_dir.to_str().unwrap());

	let file_opts = FileOptions::default().unix_permissions(0o755);

	walker.into_iter().filter_map(|entry| entry.ok()).map(|entry| {
		let path = entry.path();
		let name = path.strip_prefix(Path::new(src_dir)).unwrap();

		if path.is_file() {
			zip.start_file(name.to_str().unwrap(), file_opts)?;
			let mut buffer = Vec::new();
			eprintln!("opening {:?}", path);
			File::open(path).unwrap().read_to_end(&mut buffer).unwrap();
			zip.write_all(&*buffer).unwrap();
		} else {
			ferrisprint!("opening dir {:?}", path);
			zip.add_directory(name.to_str().unwrap(), file_opts)?;
		}
		Ok(())
	}).collect::<Result<(), _>>()
}
