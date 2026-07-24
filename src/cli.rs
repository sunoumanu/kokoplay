pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8000;
pub const DEFAULT_KOKORO_URL: &str = "http://localhost:8880/v1/audio/speech";
pub const DEFAULT_VOICE: &str = "af_heart";

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "kokoplay", version, about = "Local TTS playback service")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, default_value = DEFAULT_HOST, global = true)]
    pub host: String,

    #[arg(long, default_value_t = DEFAULT_PORT, global = true)]
    pub port: u16,

    #[arg(long, default_value = DEFAULT_KOKORO_URL, global = true)]
    pub kokoro_url: String,

    #[arg(long, default_value = DEFAULT_VOICE, global = true)]
    pub voice: String,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    /// Run the HTTP server (default when no subcommand is given)
    Serve,
    /// Speak one line of text once and exit -- no server started
    Speak { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults_apply_with_no_args() {
        let cli = Cli::try_parse_from(["kokoplay"]).unwrap();
        assert_eq!(cli.host, DEFAULT_HOST);
        assert_eq!(cli.port, DEFAULT_PORT);
        assert_eq!(cli.kokoro_url, DEFAULT_KOKORO_URL);
        assert_eq!(cli.voice, DEFAULT_VOICE);
        assert!(cli.command.is_none());
    }

    #[test]
    fn speak_subcommand_captures_text() {
        let cli = Cli::try_parse_from(["kokoplay", "speak", "hello world"]).unwrap();
        match cli.command {
            Some(Command::Speak { text }) => assert_eq!(text, "hello world"),
            other => panic!("expected Speak command, got {other:?}"),
        }
    }

    #[test]
    fn overrides_apply() {
        let cli = Cli::try_parse_from([
            "kokoplay",
            "--host",
            "0.0.0.0",
            "--port",
            "9090",
            "--kokoro-url",
            "http://example.com/speech",
            "--voice",
            "af_bella",
        ])
        .unwrap();
        assert_eq!(cli.host, "0.0.0.0");
        assert_eq!(cli.port, 9090);
        assert_eq!(cli.kokoro_url, "http://example.com/speech");
        assert_eq!(cli.voice, "af_bella");
    }
}
