use core::str;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::protocol::request::{Request, Version};
use std::str::FromStr;
use crate::protocol::response::{string_to_bytes, ResponseBuilder, ResponseCode, Response};

#[allow(dead_code, unused_variables)]
fn main() {
    // Lets say here we got the bytes from the TcpStream and we already know addr from before, where we first made contact with the client.
    let mut bytes = string_to_bytes("<CHAT \\ 1.0>\n<Method@Send>\n<Message@'Hello world!'>".to_string());
    let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());

    // Lets say that we want to encode colors in HEX in last bytes.
    // Hex color looks like this: #RRGGBB, where each R or G or B is a number in hex (between 0 and 15)
    // For each two of those hex number we need to allocate 1 byte, and to make our selfs lifes easier, we keep that '#' symbol at the -5 position and -1, to ensure that we are handling not just stray things, but our color bytes.
    
    let symb = '#' as u8; 
    bytes[512 - 1] = symb; // 512 - * is not because im dumb, its because i want to make things clearer.
    // Today I want some red, so lets use \#ff0000 (\ so Prettier want be messing with my color :) )
    bytes[512 - 4] = 255; //FF in base 10 means 255 though.
    bytes[512 - 5] = symb;

    //Now our bytes would look like this:
    // index: [... -8 -7 -6 -5  -4 -3 -2 -1]
    // bytes: [...  0  0  0 35 255  0  0 35]
    // Great!
    // As next part we would need something to process this bytes.
    // After processing bytes, we should 0 them, so our programm will continue to shine.

    let color = first_simple_bytesware(&mut bytes).unwrap(); // For now we are sure that its color.

    // Now that we parsed color out of our request line, it should be parsable.
    let send_request_line = str::from_utf8(&bytes).unwrap().trim_end_matches('\0'); //Dont forget last line! Its important!
    let mut send_request = Request::parse(send_request_line, addr.clone()).unwrap();
    // When we got our Request object, we can inject Color in it!
    send_request.varmap.insert::<Color>(color);
    send_request.varmap.insert("Jeff"); // Here we define User@Jeff, lets imagine that its already known be our thread.
    // After that we can send our request to the middleware, where we do process our request.
    // Main purpose of middleware is storing, collecting materics, validating.
    let result = middleware(send_request); 
    // Next this request should be treated by our router
    // Because in Ok() scenario this response should be sent to all listening clients
    // But in Err() scenario this response should be sent only to the requester
    let response = result.unwrap(); // But we dont care, and want only happy scenarios
    // Now that we got Response, we need to write final bytesware
    let response_bytes = last_simple_bytesware(response).unwrap(); // We still believe that we did everything correctly
    // And thats it! we made a request, that our system would handle.
}

fn first_simple_bytesware(bytes: &mut [u8; 512]) -> Result<Color, ()> {
    // Step 1: Make sure, that we are dealing with our *special* request
    let symb = '#' as u8;
    if !(bytes[512 - 1] == bytes[512 - 5] && bytes[512 - 1] == symb) {
        return Err(())
    }
    // Step 2: Extract hexnumbers
    let blue = bytes[512 - 2];
    let green = bytes[512 - 3];
    let red = bytes[512 - 4];

    let color = Color { red, green, blue };
    // Step 3: Dont forget to 0 retrieved bytes
    bytes[512-5..=512-1].fill(0);
    // Step 4: Done! Return color.
    Ok(color)
}

fn last_simple_bytesware(res: Response) -> Result<[u8; 512], ()> {
    if let Some(ref varmap) = res.varmap {
        if let Some(color) = varmap.get::<Color>() {
            let mut bytes = res.as_bytes()?;

            let symb = '#' as u8;
            bytes[512 - 1] = symb;
            bytes[512 - 2] = color.blue;
            bytes[512 - 3] = color.green;
            bytes[512 - 4] = color.red; 
            bytes[512 - 5] = symb;

            return Ok(bytes);
        }
    }

    Ok(res.as_bytes()?)
}

#[derive(Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Color {
    // Couldve implemented it from std, but im lazy lol
    pub fn to_string(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

// Something that needs to be executed between recieving the Request and sending the Response
fn middleware(req: Request) -> Result<Response, Response> {
    // This Response would be written to everybody, because its a Method@Send!
    let response = ResponseBuilder::new()
        .version(Version::CHAT10);

    if let Some(name) = req.varmap.get::<&str>() {
        if let Some(color) = req.varmap.get::<Color>() {
            let response = response
                .user(name.to_string())
                .message(req.value)
                .code(ResponseCode::OK)
                .custom_init()
                .custom_insert("Color".to_string(), color.to_string()) // Could've used this approach, just to mention
                .varmap_insert::<Color>(color.clone())
                .build()
                .unwrap(); // We believe in our self's, that we did everything right.
            //return Some(response.user(name.to_string()).message(req.value).code(ResponseCode::OK))
            return Ok(response);
        }

        // If color was not set, just dont give any!
        let response = response
            .user(name.to_string())
            .message(req.value)
            .code(ResponseCode::OK)
            .build()
            .unwrap();

        return Ok(response);
    }

    let response = response
        .code(ResponseCode::InvalidName)
        .build()
        .unwrap();

    // If we somehow forgot users name, we should write him happy letter and close connection, right?
    return Err(response);
}
