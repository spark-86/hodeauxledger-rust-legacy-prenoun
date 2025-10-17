use clap::Parser;

use crate::argv::Commands;

mod argv;
mod bootstrap;
mod find;
mod head;
mod mirror;
mod request;
mod submit;

fn print_banner() {
    println!("HodeauxLedger Usher");
    println!("===================");
}

#[tokio::main]
async fn main() {
    print_banner();
    let parsed = argv::Cli::parse();
    match parsed.command {
        Commands::Submit(submit_args) => {
            let status = submit::submit(&submit_args).await;
            match status {
                Ok(_) => {
                    println!("Success");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Commands::Request(request_args) => {
            let status = request::request(&request_args).await;
            match status {
                Ok(_) => {
                    println!("Success");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Commands::Mirror(mirror_args) => {
            let status = mirror::mirror(&mirror_args).await;
            match status {
                Ok(_) => {
                    println!("Success");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Commands::Find(find_args) => {
            let status =
                find::find_scope(&find_args, &parsed.config.unwrap(), &find_args.scope).await;
            match status {
                Ok(_) => {
                    println!("Success");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Commands::Head(head_args) => {
            let status = head::get_head(&head_args.scope, &head_args.keyfile, &head_args.cache_dir);
            match status {
                Ok(_) => {
                    println!("Success");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}
