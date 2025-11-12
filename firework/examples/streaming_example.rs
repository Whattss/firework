use firework::{get, routes, Request, Response, StatusCode};

#[get("/")]
async fn index(_req: Request, _res: Response) -> Response {
    firework::html!("<h1>Streaming Examples</h1>
    <ul>
        <li><a href=\"/stream/file\">Stream a file</a></li>
        <li><a href=\"/stream/data\">Stream generated data</a></li>
        <li><a href=\"/stream/slow\">Slow stream</a></li>
    </ul>")
}

#[get("/stream/file")]
async fn stream_file(_req: Request, _res: Response) -> Response {
    // Stream a file using tokio::fs::File
    match tokio::fs::File::open("Cargo.toml").await {
        Ok(file) => {
            firework::stream!(StatusCode::Ok, file, "text/plain")
        }
        Err(_) => {
            firework::text!(StatusCode::NotFound, "File not found")
        }
    }
}

#[get("/stream/data")]
async fn stream_data(_req: Request, _res: Response) -> Response {
    // Create an in-memory cursor to stream data
    use std::io::Cursor;
    
    let data = "This is line 1\nThis is line 2\nThis is line 3\nThis is line 4\n".repeat(100);
    let cursor = Cursor::new(data.into_bytes());
    
    firework::stream!(cursor)
}

#[get("/stream/slow")]
async fn stream_slow(_req: Request, _res: Response) -> Response {
    // Create a custom slow reader
    use tokio::io::AsyncRead;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    
    struct SlowReader {
        data: Vec<u8>,
        position: usize,
        delay: bool,
    }
    
    impl AsyncRead for SlowReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            if self.position >= self.data.len() {
                return Poll::Ready(Ok(()));
            }
            
            // Simulate slow reading by alternating between ready and pending
            if self.delay {
                self.delay = false;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            
            self.delay = true;
            
            let chunk_size = std::cmp::min(32, self.data.len() - self.position);
            let chunk = &self.data[self.position..self.position + chunk_size];
            
            buf.put_slice(chunk);
            self.position += chunk_size;
            
            Poll::Ready(Ok(()))
        }
    }
    
    let reader = SlowReader {
        data: "Slow data being streamed chunk by chunk...\n".repeat(20).into_bytes(),
        position: 0,
        delay: false,
    };
    
    firework::stream!(StatusCode::Ok, reader, "text/plain")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Streaming server running on http://localhost:8080");
    println!("Try:");
    println!("  - http://localhost:8080/");
    println!("  - http://localhost:8080/stream/file");
    println!("  - http://localhost:8080/stream/data");
    println!("  - http://localhost:8080/stream/slow");
    
    routes!().listen("127.0.0.1:8080").await?;
    Ok(())
}
