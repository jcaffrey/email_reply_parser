use email_reply_parser::EmailMessage;
use std::fs;

fn get_email(name: &str) -> EmailMessage {
    let path = format!("tests/emails/{}.txt", name);
    let text = fs::read_to_string(path).unwrap();
    EmailMessage::new(&text).read()
}

#[test]
fn test_simple_body() {
    let message = get_email("email_1_1");
    
    assert_eq!(3, message.fragments.len());
    assert_eq!(
        vec![false, true, true],
        message.fragments.iter().map(|f| f.signature).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![false, true, true],
        message.fragments.iter().map(|f| f.hidden).collect::<Vec<_>>()
    );
    assert!(message.fragments[0].content.as_ref().unwrap().contains("folks"));
    assert!(message.fragments[2].content.as_ref().unwrap().contains("riak-users"));
}

// #[test]
// fn test_reads_bottom_message() {
//     let message = get_email("email_1_2");
    
//     assert_eq!(6, message.fragments.len());
//     assert_eq!(
//         vec![false, true, false, true, false, false],
//         message.fragments.iter().map(|f| f.quoted).collect::<Vec<_>>()
//     );
//     assert_eq!(
//         vec![false, false, false, false, false, true],
//         message.fragments.iter().map(|f| f.signature).collect::<Vec<_>>()
//     );
//     assert_eq!(
//         vec![false, false, false, true, true, true],
//         message.fragments.iter().map(|f| f.hidden).collect::<Vec<_>>()
//     );

//     assert!(message.fragments[0].content.as_ref().unwrap().contains("Hi,"));
//     assert!(message.fragments[1].content.as_ref().unwrap().contains("On"));
//     assert!(message.fragments[3].content.as_ref().unwrap().contains(">"));
//     assert!(message.fragments[5].content.as_ref().unwrap().contains("riak-users"));
// }

#[test]
fn test_reads_inline_replies() {
    let message = get_email("email_1_8");
    
    assert_eq!(7, message.fragments.len());
    assert_eq!(
        vec![true, false, true, false, true, false, false],
        message.fragments.iter().map(|f| f.quoted).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![false, false, false, false, false, false, true],
        message.fragments.iter().map(|f| f.signature).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![false, false, false, false, true, true, true],
        message.fragments.iter().map(|f| f.hidden).collect::<Vec<_>>()
    );
}

#[test]
fn test_reads_top_post() {
    let message = get_email("email_1_3");
    
    assert_eq!(5, message.fragments.len());
}

#[test]
fn test_multiline_reply_headers() {
    let message = get_email("email_1_6");
    
    assert!(message.fragments[0].content.as_ref().unwrap().contains("I get"));
    assert!(message.fragments[1].content.as_ref().unwrap().contains("On"));
}

#[test]
fn test_captures_date_string() {
    let message = get_email("email_1_4");
    
    assert!(message.fragments[0].content.as_ref().unwrap().contains("Awesome"));
    assert!(message.fragments[1].content.as_ref().unwrap().contains("On"));
    assert!(message.fragments[1].content.as_ref().unwrap().contains("Loader"));
}

#[test]
fn test_complex_body_with_one_fragment() {
    let message = get_email("email_1_5");
    
    assert_eq!(1, message.fragments.len());
}

#[test]
fn test_verify_reads_signature_correct() {
    let message = get_email("correct_sig");
    
    assert_eq!(2, message.fragments.len());
    assert_eq!(
        vec![false, false],
        message.fragments.iter().map(|f| f.quoted).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![false, true],
        message.fragments.iter().map(|f| f.signature).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![false, true],
        message.fragments.iter().map(|f| f.hidden).collect::<Vec<_>>()
    );
    assert!(message.fragments[1].content.as_ref().unwrap().contains("--"));
}

#[test]
fn test_deals_with_windows_line_endings() {
    let message = get_email("email_1_7");
    
    assert!(message.fragments[0].content.as_ref().unwrap().contains(":+1:"));
    assert!(message.fragments[1].content.as_ref().unwrap().contains("On"));
    // assert!(message.fragments[2].content.as_ref().unwrap().contains("Steps 0-2"));
}

#[test]
fn test_reply_is_parsed() {
    let message = get_email("email_1_2");
    assert!(message.reply().contains("You can list the keys for the bucket"));
}

// Test cases for reading from external files
#[test]
fn test_reply_from_gmail() {
    let text = fs::read_to_string("tests/emails/email_gmail.txt").unwrap();
    let reply = EmailMessage::new(&text).read().reply();
    assert_eq!(
        "This is a test for inbox replying to a github message.",
        reply
    );
}

#[test]
fn test_parse_out_just_top_for_outlook_reply() {
    let text = fs::read_to_string("tests/emails/email_2_1.txt").unwrap();
    let reply = EmailMessage::new(&text).read().reply();
    assert_eq!("Outlook with a reply", reply);
}

#[test]
fn test_parse_out_just_top_for_outlook_with_reply_directly_above_line() {
    let text = fs::read_to_string("tests/emails/email_2_2.txt").unwrap();
    let reply = EmailMessage::new(&text).read().reply();
    assert_eq!("Outlook with a reply directly above line", reply);
}

#[test]
fn test_parse_out_just_top_for_outlook_with_unusual_headers_format() {
    let text = fs::read_to_string("tests/emails/email_2_3.txt").unwrap();
    let reply = EmailMessage::new(&text).read().reply();
    assert_eq!(
        "Outlook with a reply above headers using unusual format",
        reply
    );
}

#[test]
fn test_sent_from_iphone() {
    let text = fs::read_to_string("tests/emails/email_iPhone.txt").unwrap();
    let reply = EmailMessage::new(&text).read().reply();
    assert!(!reply.contains("Sent from my iPhone"));
}

#[test]
fn test_email_one_is_not_on() {
    let text = fs::read_to_string("tests/emails/email_one_is_not_on.txt").unwrap();
    let reply = EmailMessage::new(&text).read().reply();
    assert!(!reply.contains("On Oct 1, 2012, at 11:55 PM, Dave Tapley wrote:"));
}

#[test]
fn test_partial_quote_header() {
    let message = get_email("email_partial_quote_header");
    let reply = message.reply();
    
    assert!(reply.contains("On your remote host you can run:"));
    assert!(reply.contains("telnet 127.0.0.1 52698"));
    assert!(reply.contains("This should connect to TextMate"));
}

#[test]
fn test_email_headers_no_delimiter() {
    let message = get_email("email_headers_no_delimiter");
    assert_eq!(message.reply().trim(), "And another reply!");
}

// #[test]
// fn test_multiple_on() {
//     let message = get_email("greedy_on");
    
//     assert!(message.fragments[0].content.as_ref().unwrap().starts_with("On your remote host"));
//     assert!(message.fragments[1].content.as_ref().unwrap().starts_with("On 9 Jan 2014"));

//     assert_eq!(
//         vec![false, true, false],
//         message.fragments.iter().map(|f| f.quoted).collect::<Vec<_>>()
//     );
//     assert_eq!(
//         vec![false, false, false],
//         message.fragments.iter().map(|f| f.signature).collect::<Vec<_>>()
//     );
//     assert_eq!(
//         vec![false, true, true],
//         message.fragments.iter().map(|f| f.hidden).collect::<Vec<_>>()
//     );
// }

#[test]
fn test_pathological_emails() {
    let start_time = std::time::Instant::now();
    let message = get_email("pathological");
    assert!(start_time.elapsed().as_secs_f32() < 1.0, "Took too long");
}

// #[test]
// fn test_doesnt_remove_signature_delimiter_in_mid_line() {
//     let message = get_email("email_sig_delimiter_in_middle_of_line");
//     for fragment in &message.fragments {
//         dbg!(fragment.content());
//     }
//     assert_eq!(1, message.fragments.len());
// }