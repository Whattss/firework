use std::collections::HashMap;
use std::str::FromStr;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "OPTIONS" => Ok(Method::OPTIONS),
            "HEAD" => Ok(Method::HEAD),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::GET => "GET",
                Method::POST => "POST",
                Method::PUT => "PUT",
                Method::DELETE => "DELETE",
                Method::PATCH => "PATCH",
                Method::OPTIONS => "OPTIONS",
                Method::HEAD => "HEAD",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Route {
    pub method: Method,
    pub path: String,
}

impl Route {
    pub fn new(method: Method, path: String) -> Self {
        Route { method, path }
    }

    pub fn matches(&self, request_path: &str) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();

        let route_segments: Vec<&str> = self.path.split('/').collect();
        let request_segments: Vec<&str> = request_path.split('/').collect();

        if route_segments.len() != request_segments.len() {
            return None;  
        }

        for (route_segment, request_segment) in route_segments.iter().zip(request_segments.iter()) {
            if route_segment.starts_with(':') {
                let param_name = &route_segment[1..];
                params.insert(param_name.to_string(), request_segment.to_string());
            } else if route_segment != request_segment {
                return None; 
            }
        }

        Some(params) 
    }
}

