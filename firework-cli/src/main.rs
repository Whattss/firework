use clap::{Parser, Subcommand};

mod commands;
mod templates;

#[derive(Parser)]
#[command(name = "fwk")]
#[command(about = "Firework CLI - Build blazing fast web applications", long_about = None)]
struct Cli {
    #[arg(long, global = true, help = "Bypass Light Guard checks (sets FIREWORK_IMPURE=1)")]
    impure: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new Firework project")]
    New {
        #[arg(help = "Project name")]
        name: String,

        #[arg(short, long, help = "Use a template (basic, api, fullstack)")]
        template: Option<String>,
    },

    #[command(about = "Run in development mode with hot reload")]
    Dev,

    #[command(about = "List all registered routes")]
    Routes {
        #[arg(short, long, help = "Show only routes matching pattern")]
        filter: Option<String>,

        #[arg(short, long, help = "Show detailed route information")]
        verbose: bool,

        #[arg(short, long, help = "Export routes to format (openapi, markdown)")]
        export: Option<String>,

        #[arg(short, long, help = "Check for route conflicts")]
        check: bool,

        #[arg(short, long, help = "Show route statistics")]
        stats: bool,
    },

    #[command(about = "Create configuration file")]
    Create {
        #[command(subcommand)]
        resource: CreateResource,
    },

    #[command(about = "Run various tasks")]
    Run {
        #[command(subcommand)]
        task: RunTask,
    },
}

#[derive(Subcommand)]
enum CreateResource {
    #[command(about = "Create Firework.toml configuration")]
    Config,
}

#[derive(Subcommand)]
enum RunTask {
    #[command(about = "Run in development mode")]
    Dev {
        #[arg(long, help = "Enable hot reload")]
        hot_reload: bool,
    },

    #[command(about = "Build for release")]
    Release,

    #[command(about = "Build the project")]
    Build,

    #[command(about = "Run a custom script from Firework.toml")]
    Script {
        #[arg(help = "Script name")]
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let impure = cli.impure;

    match cli.command {
        Commands::New { name, template } => {
            commands::new_project(&name, template.as_deref());
        }
        Commands::Dev => {
            commands::run_dev(true, impure); // Always enable hot reload for dev alias
        }
        Commands::Routes {
            filter,
            verbose,
            export,
            check,
            stats,
        } => {
            commands::list_routes(filter.as_deref(), verbose, export.as_deref(), check, stats);
        }
        Commands::Create { resource } => match resource {
            CreateResource::Config => {
                commands::create_config();
            }
        },
        Commands::Run { task } => match task {
            RunTask::Dev { hot_reload } => {
                commands::run_dev(hot_reload, impure);
            }
            RunTask::Release => {
                commands::run_release(impure);
            }
            RunTask::Build => {
                commands::run_build(impure);
            }
            RunTask::Script { name } => {
                commands::run_script(&name, impure);
            }
        },
    }
}
