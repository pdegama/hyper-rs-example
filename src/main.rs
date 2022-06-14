mod server;
use server::server_listen;

fn main() {

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        server_listen(8088).await.expect("panic!");
    });

}