use argh::FromArgs;

/// Configuration
#[derive(FromArgs, Debug)]
struct Config {
    /// listen address
    #[argh(option, default = "[::1]:8080")]
    listen: String,
}

fn main() {
    let config: Config = argh::from_env();
    println!("{:?}, config")
}

//#[async_std::main]
//async fn main() -> Result<(), std::io::Error> {
//    tide::log::start();
//    let config: Config = argh::from_env();
//    let mut app = tide::new();
//
//    app.listen(config.listen).await
//}
