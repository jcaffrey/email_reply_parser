use regex::Regex;

pub struct EmailReplyParser;

impl EmailReplyParser {
    pub fn read(text: &str) -> EmailMessage {
        EmailMessage::new(text).read()
    }

    pub fn parse_reply(text: &str) -> String {
        EmailReplyParser::read(text).reply()
    }
}

pub struct EmailMessage {
    pub fragments: Vec<Fragment>,
    pub fragment: Option<Fragment>,
    pub text: String,
    pub found_visible: bool,
}

impl EmailMessage {
    pub fn new(text: &str) -> Self {
        Self {
            fragments: Vec::new(),
            fragment: None,
            text: text.replace("\r\n", "\n"),
            found_visible: false,
        }
    }

    pub fn read(mut self) -> Self {
        // Manually handle multi-quote headers
        self.text = self.fix_multi_quote_headers();
        // complex regex not supported in rust regex crate
        // let multi_quote_hdr_regex_multiline = Regex::new(
        //     r"(?s)(?!On.*On\s.+?wrote:)(On\s(.+?)wrote:)"
        // ).unwrap();
        // if let Some(captures) = multi_quote_hdr_regex_multiline.captures(&self.text) {
        //     if let Some(matched) = captures.get(0) {
        //         self.text = multi_quote_hdr_regex_multiline
        //             .replace(&self.text, matched.as_str().replace('\n', ""))
        //             .into_owned();
        //     }
        // }

        // Manually fix Outlook-style replies where the reply is above a signature boundary line
        self.text = self.fix_outlook_replies();
        // complex regex not supported in rust regex crate
        // Fix any outlook style replies, with the reply immediately above the signature boundary line
        // let outlook_reply_regex = Regex::new(r"([^\n])(?=\n ?[_-]{7,})").unwrap();
        // self.text = outlook_reply_regex.replace(&self.text, "$1\n").into_owned();

        // Collect lines and reverse them
        let lines: Vec<String> = self.text.lines().map(|line| line.to_string()).collect();

        for line in lines.iter().rev() {
            // Pass the line as a reference
            self.scan_line(line);
        }

        self.finish_fragment();
        self.fragments.reverse();

        self
    }

    pub fn reply(&self) -> String {
        self.fragments.iter()
            .filter(|f| !(f.hidden || f.quoted))
            .map(|f| f.content())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn fix_multi_quote_headers(&self) -> String {
        // This function manually processes the multi-quote headers
        let quote_header_regex = Regex::new(r"On\s(.+?)wrote:").unwrap();
        let mut result = self.text.clone();
        let mut last_pos = 0;
        
        while let Some(caps) = quote_header_regex.captures(&result[last_pos..]) {
            // let start = last_pos + caps.get(0).unwrap().start();
            let end = last_pos + caps.get(0).unwrap().end();
            // let quote_header = &result[start..end];

            // Check if there's another "On ... wrote:" within the same block
            if result[end..].contains("On ") {
                // Remove the newline between the headers
                result = result[..end].to_string() + " " + &result[end..];
            }
            
            last_pos = end;
        }
        
        result
    }

    fn fix_outlook_replies(&self) -> String {
        let lines: Vec<&str> = self.text.lines().collect();
        let mut result = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            if i + 1 < lines.len() && (lines[i + 1].trim().starts_with('_') || lines[i + 1].trim().starts_with('-')) {
                // If the next line is a signature boundary, ensure there's a newline before it
                result.push(lines[i]);
                result.push("");  // Add the missing newline
            } else {
                result.push(lines[i]);
            }
            i += 1;
        }

        result.join("\n")
    }

    fn scan_line(&mut self, line: &str) {
        let sig_regex = Regex::new(r"(--|__|-\w)|(^Sent from my (\w+\s*){1,3})").unwrap();
        let quote_hdr_regex = Regex::new(r"On.*wrote:$").unwrap();
        let quoted_regex = Regex::new(r"(>+)").unwrap();
        let header_regex = Regex::new(r"^\*?(From|Sent|To|Subject):\*? .+").unwrap();

        let is_quote_header = quote_hdr_regex.is_match(line);
        let is_quoted = quoted_regex.is_match(line);
        let is_header = is_quote_header || header_regex.is_match(line);

        if let Some(fragment) = &self.fragment {
            if fragment.lines.last().map_or(false, |last_line| sig_regex.is_match(last_line)) {
                self.fragment.as_mut().unwrap().signature = true;
                self.finish_fragment();
            }
        }

        if let Some(fragment) = &mut self.fragment {
            if fragment.headers == is_header && fragment.quoted == is_quoted ||
               fragment.quoted && (is_quote_header || line.trim().is_empty()) {
                fragment.lines.push(line.to_string());
            } else {
                self.finish_fragment();
                self.fragment = Some(Fragment::new(is_quoted, line, is_header));
            }
        } else {
            self.fragment = Some(Fragment::new(is_quoted, line, is_header));
        }
    }

    fn finish_fragment(&mut self) {
        if let Some(mut fragment) = self.fragment.take() {
            fragment.finish();
            if fragment.headers {
                self.found_visible = false;
                for frag in &mut self.fragments {
                    frag.hidden = true;
                }
            }

            if !self.found_visible {
                if fragment.quoted || fragment.headers || fragment.signature || fragment.content().trim().is_empty() {
                    fragment.hidden = true;
                } else {
                    self.found_visible = true;
                }
            }

            self.fragments.push(fragment);
        }
    }
}

pub struct Fragment {
    pub signature: bool,
    pub headers: bool,
    pub hidden: bool,
    pub quoted: bool,
    pub content: Option<String>,
    pub lines: Vec<String>,
}

impl Fragment {
    fn new(quoted: bool, first_line: &str, headers: bool) -> Self {
        Self {
            signature: false,
            headers,
            hidden: false,
            quoted,
            content: None,
            lines: vec![first_line.to_string()],
        }
    }

    fn finish(&mut self) {
        self.lines.reverse();
        self.content = Some(self.lines.join("\n").trim().to_string());
        self.lines.clear();
    }

    pub fn content(&self) -> &str {
        self.content.as_deref().unwrap_or("")
    }
}