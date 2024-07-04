mod server{
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::io::{Read, Write};
    use::std::net::TcpListener;
    use std::path::Path;


    type route = fn(&Request) -> Response;

    pub struct Request{
        method: String,
        request_uri: String,
        query_params: HashMap<String,String>,
        headers: HashMap<String,String>,
        body: String,
        file_path: String,
        route_exists: bool
    }

    impl Request{
        pub fn new() -> Request{
            Request{
                method: String::new(),
                request_uri: String::new(),
                query_params: HashMap::new(),
                headers: HashMap::new(),
                body: String::new(),
                file_path: String::new(),
                route_exists: false
            }
        }

        pub fn print(&self){
            println!("Method: {}",self.method);
            println!("Request URI: {}",self.request_uri);
            println!("Query Params: {:?}",self.query_params);
            println!("Headers: {:?}",self.headers);
            println!("Body: {}",self.body);
            println!("File Path: {}",self.file_path);
            println!("Route Exists: {}",self.route_exists);
        }

        fn parse_first_line(&mut self, line: &str){
            let parts = line.split_whitespace().collect::<Vec<&str>>();
            self.method = parts[0].to_string();
            let uri_parts = parts[1].split("?").collect::<Vec<&str>>();
            
            if uri_parts[0] == "/"{
                self.file_path = "website/index.html".to_string();
            }else if uri_parts[0].contains("."){
                self.file_path = uri_parts[0].to_string();
            }else if !uri_parts[0].contains("."){
                self.file_path = format!("website{}{}",uri_parts[0],".html");
            }

            if uri_parts.len() < 2{
                self.request_uri = uri_parts[0].to_string();
                return;
            }
            self.request_uri = uri_parts[0].to_string();
            let query_string = uri_parts[1];
            let query_parts = query_string.split("&").collect::<Vec<&str>>();
            for query_part in query_parts{
                let query_parts = query_part.split("=").collect::<Vec<&str>>();
                self.query_params.insert(query_parts[0].to_string(),query_parts[1].to_string());
            }
        }

        pub fn parse_request(&mut self, mut request_stream : &std::net::TcpStream){
            let mut buffer = [0;1024];
            request_stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..]);

            for (line_num,line) in request.lines().enumerate(){
                if line_num == 0 {
                    self.parse_first_line(line);
                    continue;
                }
                if line == ""{
                    continue;
                }

                let parts = line.split(":").collect::<Vec<&str>>();
                if parts.len() < 2{
                    // read body
                    self.body = line.to_string();
                    continue;
                }
                self.headers.insert(parts[0].to_string(),parts[1].to_string());
            }
            self.route_exists = self.route_exists();
        }
        
        pub fn route_exists(&self) -> bool{
            let path = Path::new(&self.file_path);
            path.exists()
        }
    }

    pub struct Response{
        status_code: u16,
        headers: HashMap<String,String>,
        body: String
    }

    impl Response{
        pub fn new() -> Response{
            Response{
                status_code: 1,
                headers: HashMap::new(),
                body: String::new()
            }
        }
    }

    pub fn start_server(address:String)->TcpListener {
        let listener_result = TcpListener::bind(address);
        let listener = match listener_result {
            Ok(listener) => listener,
            Err(error) => panic!("Error: {}", error),
        };
        listener
    }

    fn extract_request(stream: &std::net::TcpStream) -> Request {
        let mut request = Request::new();
        request.parse_request(stream);
        request.print();
        request
    }

    fn handle_client(mut stream: std::net::TcpStream) {
        let path = extract_request(&stream);

        let final_response = if path.route_exists {
            let contents = read_to_string(&path.file_path).unwrap();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents);
            response
        } else {
            let contents = match read_to_string("website/404.html") {
                Ok(contents) => contents,
                Err(_) => "404 Not Found".to_string(),
            };
            let response = format!("HTTP/1.1 404 NOT FOUND\r\n\r\n\r\n{}", contents);
            response
        };

        stream.write(final_response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    pub fn run_server(listener: TcpListener) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_client(stream);
                }
                Err(error) => {
                    panic!("Error: {}", error);
                }
            }
        }
    }
    

}
pub use server::*;