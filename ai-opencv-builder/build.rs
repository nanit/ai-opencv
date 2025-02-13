use std::{
	env, fs,
	path::{Path, PathBuf},
	process::Command,
};

const OPENCV_MODULES: [&str; 3] = ["imgcodecs", "imgproc", "objdetect"];

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn download_sources(source_root: &Path) -> Result<()> {
	for archive in ["opencv", "opencv_contrib"] {
		let zip_path = source_root.join(format!("{archive}.zip"));
		let status = Command::new("wget")
			.arg("-O")
			.arg(&zip_path)
			.arg(format!(
				"https://github.com/opencv/{archive}/archive/{}.zip",
				opencv_version()
			))
			.status()?;

		if !status.success() {
			return Err(format!("errors downloading {archive}: {status}").into());
		}

		let status = Command::new("unzip")
			.arg("-qq")
			.arg(&zip_path)
			.arg("-d")
			.arg(source_root)
			.status()?;
		if !status.success() {
			return Err(format!("errors unzipping {archive}: {status}").into());
		}

		fs::remove_file(zip_path)?;
	}

	Ok(())
}

fn build_opencv(install_dir: &Path, source_dir: &Path, build_dir: &Path) -> Result<()> {
	let status = Command::new("cmake")
		.args([
			"-DCMAKE_BUILD_TYPE=Release",
			"-DBUILD_SHARED_LIBS=OFF",
			"-DBUILD_DOCS=OFF",
			"-DBUILD_EXAMPLES=OFF",
			format!("-DBUILD_LIST={}", OPENCV_MODULES.join(",")).as_str(),
			"-DBUILD_opencv_apps=OFF",
			format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.to_str().unwrap()).as_str(),
			"-DBUILD_TESTS=OFF",
			"-DBUILD_PERF_TESTS=OFF",
			"-DBUILD_opencv_java=OFF",
			"-DBUILD_opencv_python=OFF",
			"-DWITH_PROTOBUF=OFF",
			"-DWITH_ADE=OFF",
			"-DBUILD_opencv_gapi=OFF",
			"-DWITH_EIGEN=OFF",
			"-DWITH_OPENEXR=OFF",
			"-DOPENCV_DNN_OPENCL=OFF",
			"-DOPENCV_FORCE_3RDPARTY_BUILD=ON",
			format!(
				"-DOPENCV_EXTRA_MODULES_PATH={}",
				source_dir
					.join(format!("opencv_contrib-{}", opencv_version()))
					.join("modules")
					.to_str()
					.unwrap()
			)
			.as_str(),
			"-S",
			source_dir.join(format!("opencv-{}", opencv_version())).to_str().unwrap(),
			"-B",
			build_dir.to_str().unwrap(),
		])
		.status()?;
	if !status.success() {
		return Err(format!("errors running cmake: {status}").into());
	}

	let status = Command::new("make")
		.arg("-C")
		.arg(build_dir.to_str().unwrap())
		.arg("-j")
		.arg("10")
		.status()?;
	if !status.success() {
		return Err(format!("errors running make: {status}").into());
	}

	let status = Command::new("make")
		.arg("-C")
		.arg(build_dir.to_str().unwrap())
		.arg("install")
		.status()?;
	if !status.success() {
		return Err(format!("errors running make install: {status}").into());
	}

	Ok(())
}

fn main() -> Result<()> {
	let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("opencv");

	if !out_dir.exists() {
		fs::create_dir(&out_dir)?;
		if let Err(e) = run_build(&out_dir) {
			fs::remove_dir_all(&out_dir)?;
			return Err(e);
		}
	}

	println!(
		"cargo::rustc-env=OPENCV_DIR={}",
		out_dir
			.join("install")
			.join("lib")
			.join("cmake")
			.join("opencv4")
			.to_str()
			.unwrap()
	);

	Ok(())
}

fn opencv_version() -> String {
	let version = env::var("CARGO_PKG_VERSION").expect("missing CARGO_PKG_VERSION env variable");
	version
		.split("opencv")
		.last()
		.expect(format!("version string: {version} is missing opencv version").as_str())
		.to_string()
}

fn run_build(out_dir: &Path) -> Result<()> {
	let source_dir = out_dir.join("source");
	fs::create_dir(&source_dir)?;

	download_sources(&source_dir)?;

	let build_dir = out_dir.join("build");
	fs::create_dir(&build_dir)?;

	let install_dir = out_dir.join("install");
	fs::create_dir(&install_dir)?;

	build_opencv(&install_dir, &source_dir, &build_dir)?;

	fs::remove_dir_all(&source_dir)?;
	fs::remove_dir_all(&build_dir)?;

	Ok(())
}
