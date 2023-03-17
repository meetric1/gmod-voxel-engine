fn main() {
	let final_dir = format!("target\\{}\\gmsv_voxels_win64.dll", std::env::var("CRATE_PROFILE").unwrap());

	// get the target dir (cursed)
	let target_dir = std::path::PathBuf::from("..\\..\\..\\lua\\bin\\gmsv_voxels_win64.dll");

	// remove & copy files
	if target_dir.exists() {std::fs::remove_file(&target_dir).unwrap()}
	std::fs::copy(final_dir, &target_dir).unwrap();
}