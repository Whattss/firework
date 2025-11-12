use clap::{Parser, Subcommand};

mod commands;
mod templates;

#[derive(Parser)]
#[command(name = "fwk")]
#[command(about = "Firework CLI - Build blazing fast web applications", long_about = None)]
struct Cli {
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
    
    match cli.command {
        Commands::New { name, template } => {
            commands::new_project(&name, template.as_deref());
        }
        Commands::Create { resource } => match resource {
            CreateResource::Config => {
                commands::create_config();
            }
        },
        Commands::Run { task } => match task {
            RunTask::Dev { hot_reload } => {
                commands::run_dev(hot_reload);
            }
            RunTask::Release => {
                commands::run_release();
            }
            RunTask::Build => {
                commands::run_build();
            }
            RunTask::Script { name } => {
                commands::run_script(&name);
            }
        },
    }
}
