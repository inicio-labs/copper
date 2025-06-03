use std::{
	collections::BTreeMap,
	env,
	fs::{self, File},
	io::{self, BufWriter, Write},
	path::{Path, PathBuf},
	process::{Command, Stdio},
	sync::LazyLock,
};

use anyhow::Context;
use prost_reflect::{
	Cardinality, DescriptorPool, ExtensionDescriptor, Kind, MessageDescriptor, Value,
};
use tonic_build::Config;
use walkdir::WalkDir;

const SIGNER_OPTION_FQN: &str = "cosmos.msg.v1.signer";

const BUF_BIN: &str = "buf";

static PROTO_SRC_DIR: LazyLock<PathBuf> = LazyLock::new(|| "proto".into());

static OUT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
	env::var("OUT_DIR")
		.inspect_err(|e| eprintln!("OUT_DIR env var must be set: {}", e))
		.unwrap()
		.into()
});

static PROTO_EXPORT_DIR: LazyLock<PathBuf> = LazyLock::new(|| OUT_DIR.join("proto_export"));
static PROTO_GEN_DIR: LazyLock<PathBuf> = LazyLock::new(|| OUT_DIR.join("proto_gen"));

#[derive(Default, Debug)]
struct ModuleNode {
	submodules: BTreeMap<String, ModuleNode>,
	src_file_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
	setup_src_watching_and_deps(&*PROTO_SRC_DIR)?;
	prepare_dir(&*PROTO_EXPORT_DIR)?;
	prepare_dir(&*PROTO_GEN_DIR)?;
	run_buf_export(&*PROTO_SRC_DIR, &*PROTO_EXPORT_DIR)?;

	let descriptor_set_path = OUT_DIR.join("file_descriptor_set.bin");
	run_buf_build_descriptor_set(&*PROTO_EXPORT_DIR, &descriptor_set_path)?;

	let (generated_trait_impls, generated_match_arms) =
		generate_all_signer_trait_impls_and_match_arms(&descriptor_set_path)?;
	write_generated_impls_to_file(
		&generated_trait_impls,
		OUT_DIR.join("generated_signer_impls.rs"),
	)?;
	write_extract_signers_from_any_msg(
		&format_extract_signers_from_any_msg(&generated_match_arms),
		OUT_DIR.join("generated_extract_signers_from_any_msg.rs"),
	)?;

	compile_protos(&*PROTO_EXPORT_DIR, &*PROTO_GEN_DIR)?;

	generate_mod_rs(&*PROTO_GEN_DIR)?;

	Ok(())
}

fn setup_src_watching_and_deps<P>(src_dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let src_dir = src_dir.as_ref();

	if !src_dir.try_exists().context(format!("failed inspecting {}", src_dir.display()))? {
		anyhow::bail!("src dir {} must exist", src_dir.display());
	}

	println!("cargo:rerun-if-changed={}", src_dir.display());
	WalkDir::new(src_dir).into_iter().try_for_each(|entry| {
		entry
			.map(|entry| {
				if entry.file_type().is_file() {
					println!("cargo:rerun-if-changed={}", entry.path().display());
				}
			})
			.inspect_err(|err| {
				eprintln!(
					"warning: error walking directory {}: {}, rerun tracking might be incomplete",
					src_dir.display(),
					err
				);
			})
	})?;

	Ok(())
}

fn prepare_dir<P>(dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let dir = dir.as_ref();
	if dir.try_exists().context(format!("failed inspecting {}", dir.display()))? {
		fs::remove_dir_all(dir)?;
	}

	fs::create_dir_all(dir)?;

	Ok(())
}

fn run_buf_export<P1, P2>(src_dir: P1, export_dir: P2) -> anyhow::Result<()>
where
	P1: AsRef<Path>,
	P2: AsRef<Path>,
{
	println!("buf export to {}", src_dir.as_ref().display());
	let buf_status = Command::new(BUF_BIN)
		.arg("export")
		.arg(src_dir.as_ref())
		.arg("--output")
		.arg(export_dir.as_ref())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()
		.context(format!(
			"{0} command failed. Ensure {0}, is installed and in PATH",
			BUF_BIN
		))?;

	if !buf_status.success() {
		anyhow::bail!("buf export failed with status: {}", buf_status);
	}

	println!("buf export finished successfully");

	Ok(())
}

fn run_buf_build_descriptor_set<P1, P2>(
	proto_dir: P1,
	descriptor_set_path: P2,
) -> anyhow::Result<()>
where
	P1: AsRef<Path>,
	P2: AsRef<Path>,
{
	let proto_dir = proto_dir.as_ref();
	let descriptor_set_path = descriptor_set_path.as_ref();
	println!(
		"buf build (for descriptor set) from '{}' to '{}'",
		proto_dir.display(),
		descriptor_set_path.display()
	);

	let buf_build_status = Command::new("buf")
		.arg("build")
		.arg(proto_dir)
		.arg("--output")
		.arg(descriptor_set_path)
		.arg("--as-file-descriptor-set")
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()
		.context("buf build (for descriptor set) command failed.")?;

	if !buf_build_status.success() {
		anyhow::bail!(
			"buf build (for descriptor set) command failed with status: {}",
			buf_build_status
		);
	}
	println!("buf build (for descriptor set) finished successfully");
	Ok(())
}

fn generate_all_signer_trait_impls_and_match_arms(
	descriptor_set_path: &Path,
) -> anyhow::Result<(String, String)> {
	let descriptor_set_bytes =
		fs::read(descriptor_set_path).context("Failed to read descriptor set")?;
	let desc_pool = DescriptorPool::decode(descriptor_set_bytes.as_slice())
		.context("Failed to decode descriptor set")?;

	let mut generated_trait_impls = String::new();
	let mut generated_match_arms = String::new();

	if let Some(signer_opt_ext_desc) = desc_pool.get_extension_by_name(SIGNER_OPTION_FQN) {
		for msg_desc in desc_pool.all_messages() {
			if let Some((impl_str, match_arm_str)) =
				try_generate_get_signers_impl_and_match_arm_for_message(
					&msg_desc,
					&signer_opt_ext_desc,
				) {
				generated_trait_impls.push_str(&impl_str);
				generated_match_arms.push_str(&match_arm_str);
			}
		}
	} else {
		eprintln!(
			"warning: extension '{}' not found in the descriptor pool, ensure its defintion is compiled",
			SIGNER_OPTION_FQN
		);
	}

	Ok((generated_trait_impls, generated_match_arms))
}

fn try_generate_get_signers_impl_and_match_arm_for_message(
	msg_desc: &MessageDescriptor,
	signer_opt_ext_desc: &ExtensionDescriptor,
) -> Option<(String, String)> {
	let message_options_dyn = msg_desc.options();

	if !message_options_dyn.has_extension(signer_opt_ext_desc) {
		return None;
	}

	match message_options_dyn.get_extension(signer_opt_ext_desc).as_ref() {
		Value::List(signer_fields_values) => {
			let full_msg_name = msg_desc.full_name();
			let rust_struct_name = msg_desc.name();

			let type_url = format!("/{}", full_msg_name);

			let package_prefix = if msg_desc.package_name().is_empty() {
				String::new()
			} else {
				format!("{}::", msg_desc.package_name().replace('.', "::"))
			};
			let rust_type_path = format!("{}{}", package_prefix, rust_struct_name);

			match extract_signer_field_names_from_list(signer_fields_values, full_msg_name) {
				Ok(signer_field_names) if !signer_field_names.is_empty() => {
					match validate_signer_fields_and_prepare_snippets(
						&signer_field_names,
						msg_desc,
						full_msg_name,
					) {
						Ok(validated_field_access_snippets) => {
							let signer_pushes_code = validated_field_access_snippets.join("\n\t\t");
							let get_signers_trait_impl_str = format_get_signers_trait_string(
								&rust_type_path,
								&signer_pushes_code,
							);

							let get_signers_match_arm_str =
								format_get_signers_match_arm_string(&rust_type_path, &type_url);

							Some((get_signers_trait_impl_str, get_signers_match_arm_str))
						},
						Err(()) => None, // Warnings already printed
					}
				},
				Ok(_) => {
					// Empty but valid list
					println!(
						"info: for message '{}', the 'cosmos.msg.v1.signer' extension list is present but empty, skipping GetSigners impl",
						full_msg_name
					);
					None
				},
				Err(()) => None, // Warnings already printed
			}
		},
		_ => None,
	}
}

fn extract_signer_field_names_from_list<'a>(
	signer_fields_values: &'a [Value],
	full_msg_name: &str,
) -> Result<Vec<&'a str>, ()> {
	let mut signer_field_names = vec![];
	let mut all_fields_are_strings = true;

	for signer_field_value in signer_fields_values {
		match signer_field_value.as_str() {
			Some(signer_field_name) => signer_field_names.push(signer_field_name),
			None => {
				eprintln!(
					"warning: for message '{}', an item in 'cosmos.msg.v1.signer' extension list is not a string, skipping GetSigners impl",
					full_msg_name
				);
				all_fields_are_strings = false;
				break;
			},
		}
	}

	if all_fields_are_strings {
		Ok(signer_field_names)
	} else {
		Err(())
	}
}

fn validate_signer_fields_and_prepare_snippets(
	signer_field_names: &[&str],
	msg_desc: &MessageDescriptor,
	full_msg_name: &str,
) -> Result<Vec<String>, ()> {
	let mut validated_field_access_snippets = vec![];
	let mut all_referenced_fields_valid = true;

	for &signer_field_name_str in signer_field_names {
		if let Some(target_field_desc) = msg_desc.get_field_by_name(signer_field_name_str) {
			match target_field_desc.kind() {
				Kind::String => match target_field_desc.cardinality() {
					Cardinality::Repeated => {
						validated_field_access_snippets.push(format!(
							"signers.extend(self.{}.iter().map(|s| s.as_str()));",
							signer_field_name_str
						));
					},
					_ => {
						validated_field_access_snippets.push(format!(
							"signers.push(self.{}.as_str());",
							signer_field_name_str
						));
					},
				},
				_ => {
					eprintln!(
						"warning: For message '{}', signer field '{}' (from 'cosmos.msg.v1.signer') is not of type string (found {:?}), skipping GetSigners impl",
						full_msg_name,
						signer_field_name_str,
						target_field_desc.kind()
					);
					all_referenced_fields_valid = false;
					break;
				},
			}
		} else {
			eprintln!(
				"warning: For message '{}', signer field '{}' (from 'cosmos.msg.v1.signer') does not exist in the message definition, skipping GetSigners impl",
				full_msg_name, signer_field_name_str
			);
			all_referenced_fields_valid = false;
			break;
		}
	}

	if all_referenced_fields_valid {
		Ok(validated_field_access_snippets)
	} else {
		Err(())
	}
}

fn format_get_signers_trait_string(rust_type_path: &str, signer_pushes_code: &str) -> String {
	format!(
		r#"
#[allow(clippy::all)] // Optional: to silence lints on generated code
impl crate::proto::GetSigners for crate::proto::{} {{
	fn signers(&self) -> Vec<&str> {{
		let mut signers = vec![];
		{}
		signers
	}}
}}
"#,
		rust_type_path, signer_pushes_code
	)
}

fn format_get_signers_match_arm_string(rust_type_path: &str, type_url: &str) -> String {
	format!(
		r#"
			"{type_url}" => msg.to_msg::<crate::proto::{rust_type_path}>().map(|m| crate::proto::GetSigners::signers(&m).iter().map(|s| s.to_string()).collect::<Vec<_>>()),
        "#,
		type_url = type_url,
		rust_type_path = rust_type_path,
	)
}

fn format_extract_signers_from_any_msg(generated_match_arms: &str) -> String {
	format!(
		r#"
use std::collections::HashSet;

pub fn extract_signers_from_any_msgs(msgs: &[cosmrs::Any]) -> anyhow::Result<HashSet<String>> {{
	let mut all_signers = HashSet::new();

	for msg in msgs {{
		let signers = match msg.type_url.as_str() {{
			{}		
			_ => continue,
		}};

		all_signers.extend(signers?);
	}}

	Ok(all_signers)
}}
"#,
		generated_match_arms
	)
}

fn write_generated_impls_to_file<P>(
	generated_trait_impls: &str,
	generated_impls_path: P,
) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let generated_impls_path = generated_impls_path.as_ref();
	let mut impls_file = BufWriter::new(File::create(generated_impls_path)?);
	writeln!(
		impls_file,
		"// This file is @generated by build.rs. Do not edit.\n{}",
		generated_trait_impls
	)
	.context("Failed to write generated signer impls")?;
	println!(
		"Generated signer trait impls at: {}",
		generated_impls_path.display()
	);
	Ok(())
}

fn write_extract_signers_from_any_msg<P>(
	generated_match_arms: &str,
	generated_match_arms_path: P,
) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let mut impls_file = BufWriter::new(File::create(generated_match_arms_path.as_ref())?);
	writeln!(
		impls_file,
		"// This file is @generated by build.rs. Do not edit.\n{}",
		generated_match_arms
	)
	.context("Failed to write generated match arms")?;
	println!(
		"Generated signer match arms at: {}",
		generated_match_arms_path.as_ref().display()
	);
	Ok(())
}

fn compile_protos<P1, P2>(proto_dir: P1, gen_dir: P2) -> anyhow::Result<()>
where
	P1: AsRef<Path>,
	P2: AsRef<Path>,
{
	let protos: Vec<_> = WalkDir::new(proto_dir.as_ref())
		.into_iter()
		.filter_map(|proto| proto.ok())
		.filter(|e| e.path().extension().is_some_and(|ext| ext == "proto"))
		.map(|e| e.path().to_path_buf())
		.collect();

	if protos.is_empty() {
		println!(
			"cargo:warning=no .proto files found in '{}'",
			proto_dir.as_ref().display()
		);
	}

	println!("running tonic-build");
	Config::new()
		.out_dir(gen_dir.as_ref())
		.enable_type_names()
		.compile_protos(&protos, &[proto_dir])
		.context("tonic_build compilation failed")?;

	println!("tonic-build finished successfully");
	Ok(())
}

fn insert_into_tree(root: &mut ModuleNode, parts: &[String], original_file_name: String) {
	let mut current_node = root;
	for part in parts {
		current_node = current_node.submodules.entry(part.clone()).or_default();
	}
	current_node.src_file_name = Some(original_file_name.to_string());
}

fn write_tree_to_mod_rs_recursive(
	writer: &mut BufWriter<File>,
	module_map: &BTreeMap<String, ModuleNode>,
	indent_level: usize,
	current_path_segments: &mut Vec<String>,
) -> io::Result<()> {
	let indent = "\t".repeat(indent_level);

	for (mod_segment_name, node_data) in module_map {
		current_path_segments.push(mod_segment_name.clone());
		let rust_mod_name = mod_segment_name.to_lowercase();

		writeln!(writer, "{}pub mod {} {{", indent, rust_mod_name)?;

		match node_data.src_file_name.as_ref() {
			Some(src_file_name) => {
				writeln!(
					writer,
					"{}include!(\"{}\");",
					"\t".repeat(indent_level + 1),
					src_file_name
				)?;
			},
			None => {
				println!(
					"mod.rs entry for module path '{}': pub mod {} {{ ... }} (namespace only)",
					current_path_segments.join("::"),
					rust_mod_name
				);
			},
		}

		// Recursively define submodules, if any, within the current module block.
		// This handles cases where a package like "a.b" might have generated "a.b.rs"
		// AND there are further sub-packages like "a.b.c" which generates "a.b.c.rs".
		// The "a.b.c.rs" content would be included inside the "c" module, within "b".
		if !node_data.submodules.is_empty() {
			write_tree_to_mod_rs_recursive(
				writer,
				&node_data.submodules,
				indent_level + 1,
				current_path_segments,
			)?;
		}

		writeln!(writer, "{}}}\n", indent)?; // Close the current module block
		current_path_segments.pop();
	}

	Ok(())
}

fn generate_mod_rs<P>(gen_dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let mut root_module_node = ModuleNode::default();
	let mut found_rs_files_count = 0;

	let gen_dir = gen_dir.as_ref();

	let mod_rs_path = gen_dir.join("mod.rs");
	let mut mod_rs_file = BufWriter::new(File::create(&mod_rs_path)?);

	if !gen_dir.try_exists()? {
		println!(
			"cargo:warning=PROTO_GEN_DIR ({}) does not exist, generated mod.rs will be empty regarding protos",
			gen_dir.display()
		);

		writeln!(
			mod_rs_file,
			"// no Protobuf Rust files found in PROTO_GEN_DIR ({}) during build",
			gen_dir.display()
		)?;

		mod_rs_file.flush()?;

		return Ok(());
	}

	println!(
		"scanning PROTO_GEN_DIR ({}) for generated .rs files to build mod.rs...",
		gen_dir.display()
	);
	for entry in fs::read_dir(gen_dir)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_file() {
			if let Some(extension) = path.extension() {
				if extension == "rs" {
					let original_file_name = path
						.file_name()
						.with_context(|| {
							format!("Failed to get file name for path: {}", path.display())
						})?
						.to_str()
						.with_context(|| {
							format!("File name is not valid UTF-8: {}", path.display())
						})?;

					if original_file_name == "mod.rs" {
						continue;
					}
					found_rs_files_count += 1;

					// package parts from filename (e.g., "cosmos.bank.v1beta1.rs" -> ["cosmos", "bank", "v1beta1"])
					let package_string = original_file_name.trim_end_matches(".rs");
					let parts: Vec<String> = package_string.split('.').map(String::from).collect();

					if !parts.is_empty() {
						println!(
							"processing for mod.rs: file='{}', package_parts='{:?}'",
							original_file_name, parts
						);
						insert_into_tree(&mut root_module_node, &parts, original_file_name.into());
					} else {
						println!(
							"cargo:warning=skipping file for mod.rs (no package parts derived): '{}'",
							original_file_name
						);
					}
				}
			}
		}
	}

	writeln!(
		mod_rs_file,
		"// This file is @generated by build.rs. Do not edit directly."
	)?;
	if found_rs_files_count == 0 {
		println!(
			"cargo:warning=No .rs files (other than potential mod.rs itself) found in {} to include in generated mod.rs. This might be unexpected if protos were supposed to be compiled.",
			gen_dir.display()
		);
		writeln!(
			mod_rs_file,
			"// No Protobuf Rust files were found in PROTO_GEN_DIR during build."
		)?;
	} else {
		writeln!(
			mod_rs_file,
			"// It includes all Rust modules generated from .proto files, structured hierarchically using include!."
		)?;
		writeln!(mod_rs_file)?;
		let mut current_path_segments = vec![];
		write_tree_to_mod_rs_recursive(
			&mut mod_rs_file,
			&root_module_node.submodules,
			0,
			&mut current_path_segments,
		)?;
	}

	mod_rs_file.flush()?;

	println!(
		"generated nested mod.rs with include! strategy at {}",
		mod_rs_path.display()
	);

	Ok(())
}
