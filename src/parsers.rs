use std;

pub fn split(to_split: &str, by: &str, split_limit: Option<usize>, expected_length: Option<usize>) -> Result<Vec<String>, std::io::Error> {
	// Split elements
	let mut splitted = Vec::new();
	if let Some(limit) = split_limit { to_split.splitn(limit, by).for_each(|e| splitted.push(e.to_owned())) }
		else { to_split.split(by).for_each(|e| splitted.push(e.to_owned())) };
	
	// Validate count
	if let Some(expected_length) = expected_length {
		if expected_length != splitted.len() { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) }
	}
	Ok(splitted)
}


pub mod http_header {
	use std;
	
	fn parse_lines(request: &str) -> Result<Vec<String>, std::io::Error> {
		let splitted = super::split(request, "\r\n", None, None)?;
		if splitted.len() < 1 { Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) }
			else { Ok(splitted) }
	}
	
	fn parse_header_line(header_line: &str) -> Result<(String, String, String), std::io::Error> {
		let elements = super::split(header_line, " ", Some(3), Some(3))?;
		Ok((elements[0].clone(), elements[1].clone(), elements[2].clone()))
	}
	
	fn parse_request_line(request_line: &str) -> Result<(String, String), std::io::Error> {
		let elements = super::split(request_line, ": ", Some(2), Some(2))?;
		Ok((elements[0].clone(), elements[1].clone()))
	}
	
	pub fn parse(request: Vec<u8>) -> Result<((String, String, String), std::collections::HashMap<String, String>), std::io::Error> {
		// Parse request-string and split it into lines
		let request = match String::from_utf8(request) {
			Ok(string) => string.trim().to_owned(),
			Err(_) => { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) }
		};
		// Get request lines
		let request_lines = parse_lines(&request)?;
		// Parse header-line
		let header_line = parse_header_line(&request_lines[0])?;
		
		// Parse header-fields
		let mut header_fields = std::collections::HashMap::<String, String>::new();
		for request_line in request_lines.iter().skip(1) {
			let parsed = parse_request_line(request_line)?;
			header_fields.insert(parsed.0, parsed.1);
		}
		
		Ok((header_line, header_fields))
	}
}

