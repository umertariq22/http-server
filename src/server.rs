mod server{
    use std::fs::read_to_string;
    use std::io::{Read, Write};
    use::std::net::TcpListener;
    use std::path::Path;

    fn route_exists(path: &str) -> bool {
        let complete_path = format!("website/app{}.html", path);
        let path_buf = Path::new(complete_path.as_str());
        path_buf.exists()
    }
    pub fn start_server(address:String)->TcpListener {
        let listener = TcpListener::bind(address).unwrap();
        println!("Server listening on port 8080");
        listener
    }
    fn extract_path(request: &str) -> &str {
        let request_line = request.lines().next().unwrap();
        let request_line = request_line.split_whitespace().collect::<Vec<_>>();
        let path = request_line[1];
        path
    }
    fn generate_response(path:&str) -> String {
        let mut is_html = true;
        let mut file_path = path;
        println!("path: {}", path);
        if path == "/"{
            file_path = "/index";
        }
        if path.contains("."){
            let path_split: Vec<&str> = path.split('.').collect();
            let extension = path_split[path_split.len() - 1];
            if extension != "html"{
                is_html = false;
            }else{
                file_path = path_split[0]; 
            }
        }
        println!("file_path: {}", file_path);
        let response = if is_html{
            if route_exists(file_path){
                let complete_path = format!("website/app{}.html", file_path);
                println!("complete_path: {}", complete_path);
                let file_content = read_to_string(complete_path).unwrap();
                format!("HTTP/1.1 200 OK\r\n\r\n{}", file_content)
            }else{
                let file_content = read_to_string("website/app/404.html").unwrap();
                format!("HTTP/1.1 404 NOT FOUND\r\n\r\n{}", file_content)
            }
        }else{
            let file_content = read_to_string(format!("website/{}", path)).unwrap();
            format!("HTTP/1.1 200 OK\r\n\r\n{}", file_content)
        };
        response    
    }
    pub fn handle_requests(listener:&TcpListener){
        let mut buffer = [0; 1024];
        for stream in listener.incoming(){
            let mut stream = stream.unwrap();
            stream.read(&mut buffer).unwrap();
            let path = extract_path(std::str::from_utf8(&buffer).unwrap());
            let response = generate_response(path);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
pub use server::*;