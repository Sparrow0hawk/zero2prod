use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run("127.0.0.1:8000")?.await
}
