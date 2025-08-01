use app::consensus::{
	config::Config,
	node::{CopperNode, ProtobufCodec},
	types::ConsensusHeight,
};
use malachitebft_app_channel::app::node::Node;
use malachitebft_test_cli::{
	args::{Args, Commands},
	cmd::{dump_wal::DumpWalCmd, init::InitCmd, start::StartCmd, testnet::TestnetCmd},
	config::{LogFormat, LogLevel},
	logging, runtime,
};

fn main() -> eyre::Result<()> {
	color_eyre::install()?;

	// Load command-line arguments and possible configuration file.
	let args = Args::new();

	// Parse the input command.
	match &args.command {
		Commands::Start(cmd) => start(&args, cmd),
		Commands::Init(cmd) => init(&args, cmd),
		Commands::Testnet(cmd) => testnet(&args, cmd),
		Commands::DumpWal(cmd) => dump_wal(&args, cmd),
		Commands::DistributedTestnet(_) => unimplemented!(),
	}
}

fn start(args: &Args, cmd: &StartCmd) -> eyre::Result<()> {
	// Setup the application
	let app = CopperNode {
		home_dir: args.get_home_dir()?,
		config_file: args.get_config_file_path()?,
		genesis_file: args.get_genesis_file_path()?,
		private_key_file: args.get_priv_validator_key_file_path()?,
		start_height: cmd.start_height.map(ConsensusHeight::new),
	};

	let config: Config = app.load_config()?;

	// This is a drop guard responsible for flushing any remaining logs when the program terminates.
	// It must be assigned to a binding that is not _, as _ will result in the guard being dropped immediately.
	let _guard = logging::init(config.logging.log_level, config.logging.log_format);

	let rt = runtime::build_runtime(config.runtime)?;

	tracing::info!(moniker = %config.moniker, "starting Malachite");

	// Start the node
	rt.block_on(app.run()).map_err(|e| eyre::eyre!("failed to run the application node: {e}"))
}

fn init(args: &Args, cmd: &InitCmd) -> eyre::Result<()> {
	// This is a drop guard responsible for flushing any remaining logs when the program terminates.
	// It must be assigned to a binding that is not _, as _ will result in the guard being dropped immediately.
	let _guard = logging::init(LogLevel::Info, LogFormat::Plaintext);

	// Setup the application
	let app = CopperNode {
		home_dir: args.get_home_dir()?,
		config_file: args.get_config_file_path()?,
		genesis_file: args.get_genesis_file_path()?,
		private_key_file: args.get_priv_validator_key_file_path()?,
		start_height: None,
	};

	cmd.run(
		&app,
		&args.get_config_file_path()?,
		&args.get_genesis_file_path()?,
		&args.get_priv_validator_key_file_path()?,
	)
	.map_err(|e| eyre::eyre!("failed to run init command {e:?}"))
}

fn testnet(args: &Args, cmd: &TestnetCmd) -> eyre::Result<()> {
	// This is a drop guard responsible for flushing any remaining logs when the program terminates.
	// It must be assigned to a binding that is not _, as _ will result in the guard being dropped immediately.
	let _guard = logging::init(LogLevel::Info, LogFormat::Plaintext);

	// Setup the application
	let app = CopperNode {
		home_dir: args.get_home_dir()?,
		config_file: args.get_config_file_path()?,
		genesis_file: args.get_genesis_file_path()?,
		private_key_file: args.get_priv_validator_key_file_path()?,
		start_height: Some(ConsensusHeight::new(1)), // We always start at height 1
	};

	cmd.run(&app, &args.get_home_dir()?)
		.map_err(|e| eyre::eyre!("Failed to run testnet command {e:?}"))
}

fn dump_wal(_args: &Args, cmd: &DumpWalCmd) -> eyre::Result<()> {
	// This is a drop guard responsible for flushing any remaining logs when the program terminates.
	// It must be assigned to a binding that is not _, as _ will result in the guard being dropped immediately.
	let _guard = logging::init(LogLevel::Info, LogFormat::Plaintext);

	cmd.run(ProtobufCodec).map_err(|e| eyre::eyre!("Failed to run dump-wal command {e:?}"))
}
