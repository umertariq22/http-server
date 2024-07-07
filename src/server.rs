mod server{
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use::std::net::TcpListener;
    use std::path::Path;


    type Route = fn(&Request) -> Response;

    pub struct Router{
        routes: HashMap<String,Route>
    }

    impl Router{
        pub fn new() -> Router{
            Router{
                routes: HashMap::new()
            }
        }

        pub fn add_route(&mut self, path: String, route: Route){
            self.routes.insert(path,route);
        }

        pub fn get_route(&self, path: &String) -> Option<&Route>{
            self.routes.get(path)
        }

        pub fn delete_route(&mut self, path: &String){
            self.routes.remove(path);
        }
    
    }

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

        pub fn set_status_code(&mut self, status_code: u16){
            self.status_code = status_code;
        }

        pub fn set_header(&mut self, key: String, value: String){
            self.headers.insert(key,value);
        }

        pub fn set_body(&mut self, body: String){
            self.body = body;
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

    fn create_response_string(response: &Response) -> String {
        let mut response_string = format!("HTTP/1.1 {}\r\n", response.status_code);
        for (key, value) in &response.headers {
            response_string.push_str(&format!("{}: {}\r\n", key, value));
        }
        response_string.push_str("\r\n");
        response_string.push_str(&response.body);
        response_string
    }

    fn handle_client(mut stream: std::net::TcpStream, router: &mut Router) {
        let path = extract_request(&stream);

        let route_exists = path.route_exists;
        if !route_exists{
            let new_route = "404".to_string();
            if router.get_route(&new_route).is_none(){
                router.add_route(new_route.clone(),|path|{
                    let mut response = Response::new();
                    response.set_status_code(404);
                    response.set_body("404 Not Found".to_string());
                    response
                });
            }


        }
        
        let route = router.get_route(&path.request_uri).unwrap();
        let response = (route)(&path);

        let final_response = create_response_string(&response);

        stream.write(final_response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    pub fn listen(listener: TcpListener, router: &mut Router) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_client(stream,router);
                }
                Err(error) => {
                    panic!("Error: {}", error);
                }
            }
        }
    }
    

}
pub use server::*;