mod audio;
mod cli;
mod http;
mod kokoro;
mod tts;
mod worker;

use clap::Parser;

fn main() -> std::process::ExitCode {
    let cli = cli::Cli::parse();
    match cli.command.clone().unwrap_or(cli::Command::Serve) {
        cli::Command::Serve => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build tokio runtime");
            rt.block_on(run_server(cli))
        }
        cli::Command::Speak { text } => run_speak_once(&cli, &text),
    }
}

fn run_speak_once(cli: &cli::Cli, text: &str) -> std::process::ExitCode {
    let client = kokoro::KokoroClient::new(cli.kokoro_url.clone(), cli.voice.clone());
    match tts::speak(&client, text) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("speak failed: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run_server(cli: cli::Cli) -> std::process::ExitCode {
    let client = kokoro::KokoroClient::new(cli.kokoro_url.clone(), cli.voice.clone());
    let handle = worker::spawn_worker(move |text| {
        tts::speak(&client, text).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    });

    let state = http::AppState {
        tx: handle.tx,
        queue_len: handle.queue_len,
    };
    let app = http::build_router(state);

    let addr = format!("{}:{}", cli.host, cli.port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to bind {addr}: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    println!("kokoplay listening on http://{addr}");
    match axum::serve(listener, app).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("server error: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
