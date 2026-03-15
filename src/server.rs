use serde::{Deserialize, Serialize};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use xplm::debugln;

use crate::config::ServerConfig;
use crate::ref_actions::{DataValue, RefActions};

pub struct Server {
    pub port: u16,
    pub address: String,
    ref_actions: RefActions,
}

struct Request {
    pub method: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
struct GenericResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
struct HasRefResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub has_ref: bool,
    pub is_writeable: bool,
}

#[derive(Serialize, Deserialize)]
struct GetRefResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub value: DataValue,
}

// #[derive(Serialize, Deserialize)]
// struct SetRefRequest {
//     pub value: DataValue,
// }

const OK_RESPONSE: &str = "HTTP/1.1 200 OK";
const BAD_REQUEST_RESPONSE: &str = "HTTP/1.1 400 Bad Request";
const INTERNAL_SERVER_ERROR_RESPONSE: &str = "HTTP/1.1 500 Internal Server Error";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found";

#[allow(dead_code)]
enum Response {
    Ok,
    BadRequest,
    InternalServerError,
    NotFound,
}

impl Server {
    pub fn new(config: ServerConfig, ref_actions: RefActions) -> Self {
        Self {
            port: config.port,
            address: config.address,
            ref_actions: ref_actions,
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port)).unwrap();

        debugln!("XPHTTPBridge: Listening on {}:{}", self.address, self.port);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            debugln!("XPHTTPBridge: New connection");
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, stream: std::net::TcpStream) {
        let mut buf_reader = BufReader::new(&stream);
        let request = self.get_request_type(&mut buf_reader);

        if let Some(req) = request {
            match req.method.as_str() {
                "GET" => {
                    debugln!("XPHTTPBridge: GET request: {}", req.path);
                    let resp_res = self.handle_get(req, stream);
                    match resp_res {
                        Ok(_) => {}
                        Err(err) => {
                            debugln!("XPHTTPBridge: Request error: {}", err);
                        }
                    }
                }
                "POST" => {
                    debugln!("XPHTTPBridge: POST request: {}", req.path);

                    let resp_res = self.handle_post(req, stream);
                    match resp_res {
                        Ok(_) => {}
                        Err(err) => {
                            debugln!("XPHTTPBridge: Request error: {}", err);
                        }
                    }
                }
                _ => {
                    debugln!("XPHTTPBridge: Unsupported method: {}", req.method);
                    let res = GenericResponse {
                        status: 400,
                        message: "Bad Request".to_string(),
                    };
                    let json_res = serde_json::to_string(&res);

                    let json = match json_res {
                        Ok(v) => v,
                        Err(_) => return,
                    };
                    let write_res = self.write_response(stream, Response::BadRequest, json);
                    match write_res {
                        Ok(_) => {}
                        Err(err) => {
                            debugln!("XPHTTPBridge: Failed to write response: {}", err);
                        }
                    }
                }
            }
        }
    }

    fn get_request_type(
        &self,
        buf_reader: &mut BufReader<&std::net::TcpStream>,
    ) -> Option<Request> {
        let request_line_res = buf_reader.lines().next();

        let request_line = match request_line_res {
            Some(Ok(line)) => line,
            _ => {
                return None;
            }
        };

        let mut reqeust_parts = request_line.split_whitespace();

        let method = reqeust_parts.next().unwrap_or("").to_string();
        let path = reqeust_parts.next().unwrap_or("").to_string();

        if method.is_empty() || path.is_empty() {
            return None;
        }

        Some(Request { method, path })
    }

    fn handle_get(&self, req: Request, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        match req.path.as_str() {
            "/ping" => self.handle_ping(stream),
            p if p.starts_with("/refs") => self.handle_refs(req, stream),
            _ => {
                let res = GenericResponse {
                    status: 404,
                    message: "Not Found".to_string(),
                };
                let json_res = serde_json::to_string(&res);

                let json = match json_res {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };
                self.write_response(stream, Response::NotFound, json)
            }
        }
    }

    fn handle_post(&self, req: Request, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        match req.path.as_str() {
            p if p.starts_with("/refs") => self.handle_refs_post(req, stream),
            _ => {
                let res = GenericResponse {
                    status: 404,
                    message: "Not Found".to_string(),
                };
                let json_res = serde_json::to_string(&res);

                let json = match json_res {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };
                self.write_response(stream, Response::NotFound, json)
            }
        }
    }

    fn write_response(
        &self,
        mut stream: std::net::TcpStream,
        response_type: Response,
        body: String,
    ) -> Result<(), std::io::Error> {
        let mut response_line: Vec<&[u8]> = Vec::new();

        let status_line = match response_type {
            Response::Ok => OK_RESPONSE,
            Response::BadRequest => BAD_REQUEST_RESPONSE,
            Response::InternalServerError => INTERNAL_SERVER_ERROR_RESPONSE,
            Response::NotFound => NOT_FOUND_RESPONSE,
        };

        let content_length = body.len();
        let content_length_str = content_length.to_string();

        response_line.push(status_line.as_bytes());
        response_line.push(b"\r\n");
        response_line.push(b"Content-Length: ");
        response_line.push(content_length_str.as_bytes());
        response_line.push(b"\r\n\r\n");
        response_line.push(body.as_bytes());

        let response = response_line.concat();
        let err = stream.write_all(&response).map_err(|e| e);
        return err;
    }

    // Begin handlers
    fn handle_ping(&self, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        let ping_res = GenericResponse {
            status: 200,
            message: "pong".to_string(),
        };
        let json_res = serde_json::to_string(&ping_res);

        let json = match json_res {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        self.write_response(stream, Response::Ok, json)
    }

    fn handle_refs(&self, req: Request, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        let refs_subpath = req.path.strip_prefix("/refs/");

        if refs_subpath.is_none() {
            let ping_res = GenericResponse {
                status: 400,
                message: "missing ref subpath".to_string(),
            };
            let json_res = serde_json::to_string(&ping_res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::BadRequest, json);
        }

        let refs_subpath = refs_subpath.unwrap();

        match refs_subpath {
            p if p.starts_with("has/") => self.has_ref(req, stream),
            p if p.starts_with("get/") => self.get_ref(req, stream),
            _ => {
                let ping_res = GenericResponse {
                    status: 400,
                    message: "invalid ref subpath".to_string(),
                };
                let json_res = serde_json::to_string(&ping_res);

                let json = match json_res {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };

                self.write_response(stream, Response::BadRequest, json)
            }
        }
    }

    fn handle_refs_post(
        &self,
        req: Request,
        stream: std::net::TcpStream,
    ) -> Result<(), std::io::Error> {
        let refs_subpath = req.path.strip_prefix("/refs/");

        if refs_subpath.is_none() {
            let ping_res = GenericResponse {
                status: 400,
                message: "missing ref subpath".to_string(),
            };
            let json_res = serde_json::to_string(&ping_res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::BadRequest, json);
        }

        let refs_subpath = refs_subpath.unwrap();

        match refs_subpath {
            p if p.starts_with("set/") => self.set_ref(req, stream),
            _ => {
                let ping_res = GenericResponse {
                    status: 400,
                    message: "invalid ref subpath".to_string(),
                };
                let json_res = serde_json::to_string(&ping_res);

                let json = match json_res {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };

                self.write_response(stream, Response::BadRequest, json)
            }
        }
    }

    fn has_ref(&self, req: Request, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        let ref_name = req.path.strip_prefix("/refs/has/");

        if ref_name.is_none() {
            let ping_res = GenericResponse {
                status: 400,
                message: "missing ref name".to_string(),
            };
            let json_res = serde_json::to_string(&ping_res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::BadRequest, json);
        }

        // we can safely unwrap here since we checked for None above
        let ref_name = ref_name.unwrap();
        let exists = self.ref_actions.has_ref(ref_name);

        if !exists {
            let res = HasRefResponse {
                status: 404,
                message: "ref not found".to_string(),
                ref_name: ref_name.to_string(),
                has_ref: false,
                is_writeable: false,
            };

            let json_res = serde_json::to_string(&res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::NotFound, json);
        }

        let is_writeable = self.ref_actions.is_writeable(ref_name);

        let res = HasRefResponse {
            status: 200,
            message: "ref found".to_string(),
            ref_name: ref_name.to_string(),
            has_ref: true,
            is_writeable,
        };

        let json_res = serde_json::to_string(&res);

        let json = match json_res {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        if exists {
            return self.write_response(stream, Response::Ok, json);
        }

        return self.write_response(stream, Response::NotFound, json);
    }

    fn get_ref(&self, req: Request, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        let ref_name = req.path.strip_prefix("/refs/get/");

        if ref_name.is_none() {
            let ping_res = GenericResponse {
                status: 400,
                message: "missing ref name".to_string(),
            };
            let json_res = serde_json::to_string(&ping_res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::BadRequest, json);
        }

        // we can safely unwrap here since we checked for None above
        let ref_name = ref_name.unwrap();
        let exists = self.ref_actions.has_ref(ref_name);

        if !exists {
            let res = GetRefResponse {
                status: 404,
                message: "ref not found".to_string(),
                ref_name: ref_name.to_string(),
                value: DataValue::Bool(false),
            };

            let json_res = serde_json::to_string(&res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::NotFound, json);
        }

        let ref_value_res = self.ref_actions.get_ref(ref_name);

        let ref_value = match ref_value_res {
            Some(v) => v,
            None => {
                let res = GetRefResponse {
                    status: 500,
                    message: "internal error".to_string(),
                    ref_name: ref_name.to_string(),
                    value: DataValue::Bool(false),
                };

                let json_res = serde_json::to_string(&res);

                let json = match json_res {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };

                return self.write_response(stream, Response::InternalServerError, json);
            }
        };

        let res = GetRefResponse {
            status: 200,
            message: "ok".to_string(),
            ref_name: ref_name.to_string(),
            value: ref_value,
        };

        let json_res = serde_json::to_string(&res);

        let json = match json_res {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        return self.write_response(stream, Response::Ok, json);
    }

    fn set_ref(&self, req: Request, stream: std::net::TcpStream) -> Result<(), std::io::Error> {
        let ref_name = req.path.strip_prefix("/refs/set/");

        if ref_name.is_none() {
            let ping_res = GenericResponse {
                status: 400,
                message: "missing ref name".to_string(),
            };
            let json_res = serde_json::to_string(&ping_res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::BadRequest, json);
        }

        // we can safely unwrap here since we checked for None above
        let ref_name = ref_name.unwrap();
        let exists = self.ref_actions.has_ref(ref_name);

        if !exists {
            let res = GetRefResponse {
                status: 404,
                message: "ref not found".to_string(),
                ref_name: ref_name.to_string(),
                value: DataValue::Bool(false),
            };

            let json_res = serde_json::to_string(&res);

            let json = match json_res {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            return self.write_response(stream, Response::NotFound, json);
        }

        // For testing, body handling is missing right now
        let write_ok = self
            .ref_actions
            .write_ref("sim/cockpit/electrical/battery_on", DataValue::I32(1));

        let res: GenericResponse;

        if write_ok {
            res = GenericResponse {
                status: 200,
                message: "ok".to_string(),
            };
        } else {
            res = GenericResponse {
                status: 500,
                message: "internal server error".to_string(),
            };
        }

        let json_res = serde_json::to_string(&res);

        let json = match json_res {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        if write_ok {
            return self.write_response(stream, Response::Ok, json);
        }

        return self.write_response(stream, Response::InternalServerError, json);
    }
}
