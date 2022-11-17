mod server;
use server::server_listen;

fn main() {

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        server_listen(8080).await;
    });

}

