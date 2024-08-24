use regex::Regex;
use fancy_regex::{Regex as FancyRegex};


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

        let multi_quote_hdr_regex_multiline = FancyRegex::new(
            r"(?s)(?!On.*On\s.+?wrote:)(On\s(.+?)wrote:)"
        ).unwrap();
        if let Some(captures) = multi_quote_hdr_regex_multiline.captures(&self.text).ok() {
            if let Some(matched) = captures.as_ref().and_then(|c| c.get(0)) {
                self.text = multi_quote_hdr_regex_multiline
                    .replace(&self.text, matched.as_str().replace('\n', ""))
                    .into_owned();
            }
        }


        // Fix any outlook style replies, with the reply immediately above the signature boundary line
        let outlook_reply_regex = FancyRegex::new(r"([^\n])(?=\n ?[_-]{7,})").unwrap();
        self.text = outlook_reply_regex.replace(&self.text, "$1\n").into_owned();

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

    fn scan_line(&mut self, line: &str) {
        let sig_regex = Regex::new(r"(--|__|-\w)|(^Sent from my (\w+\s*){1,3})").unwrap();
        let quote_hdr_regex = Regex::new(r"On.*wrote:$").unwrap();
        let quoted_regex = Regex::new(r"(>+)").unwrap();
        let header_regex = Regex::new(r"^\*?(From|Sent|To|Subject):\*? .+").unwrap();
        let forwarded_msg_regex = Regex::new(r"^[-]+ Forwarded message [-]+$").unwrap();

        let is_quote_header = quote_hdr_regex.is_match(line);
        let is_quoted = quoted_regex.is_match(line);
        let is_header = is_quote_header || header_regex.is_match(line) || forwarded_msg_regex.is_match(line);

        if let Some(fragment) = &self.fragment {
            if line.trim().is_empty() {
                if fragment.lines.last().map_or(false, |last_line| sig_regex.is_match(last_line.trim())) {
                    self.fragment.as_mut().unwrap().signature = true;
                    self.finish_fragment();
                }
            }
        }

        if let Some(fragment) = &mut self.fragment {
            if (fragment.headers == is_header && fragment.quoted == is_quoted) ||
               (fragment.quoted && (is_quote_header || line.trim().is_empty())) {
                fragment.lines.push(line.to_string());
            } else {
                self.finish_fragment();
                self.fragment = Some(Fragment::new(is_quoted, line, is_header));
            }
        } else {
            self.fragment = Some(Fragment::new(is_quoted, line, is_header));
        }
        // if let Some(fragment) = &mut self.fragment {
        //     if fragment.headers == is_header && fragment.quoted == is_quoted ||
        //        fragment.quoted && (is_quote_header || line.trim().is_empty()) {
        //         fragment.lines.push(line.to_string());
        //     } else {
        //         self.finish_fragment();
        //         self.fragment = Some(Fragment::new(is_quoted, line, is_header));
        //     }
        // } else {
        //     self.fragment = Some(Fragment::new(is_quoted, line, is_header));
        // }
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
        let mut content = self.lines.join("\n").trim().to_string();
        if content.ends_with("wrote:") {
            content = content.trim_end_matches("wrote:").trim_end().to_string();
        }
        self.content = Some(content);        self.lines.clear();
    }

    pub fn content(&self) -> &str {
        self.content.as_deref().unwrap_or("").trim()
    }
}