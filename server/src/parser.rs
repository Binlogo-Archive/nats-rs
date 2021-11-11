/*
Protocol definition: https://docs.nats.io/nats-protocol/nats-protocol

- Subject names and wildcards
    - Subjects are case-sensitive and cannot contain whitespace.
    - The `.` character is used to create a subject hierarchy.
```
time.us
time.us.east
time.us.east.atlanta
time.eu.east
time.eu.warsaw
```

- Field Delimiters
    - '`'\(space\) or\t` (tab). Multiple whitespace characters will be treated as a single field delimiter.

- Newlines
    - NATS uses CR followed by LF (CR+LF, \r\n, 0x0D0A) to terminate protocol messages.

- Messages
    - NATS protocol operation names are case insensitive, thus `SUB foo 1\r\n` and `sub foo 1\r\n` are equivalent.

## PUB
```
PUB <subject> [reply-to] <#bytes>\r\n[payload]\r
```
## SUB
```
SUB <subject> [queue group] <sid>\r
```
## MSG
```
MSG <subject> <sid> [reply-to] <#bytes>\r\n[payload]\r
```
 */

use crate::error::*;

macro_rules! parse_error {
    () => {{
        return Err(NError::new(ERROR_PARSE));
    }};
}

#[derive(Debug, Clone)]
enum ParseState {
    OpStart,
    OpP,
    OpPu,
    OpPub,
    OpPubSpace,
    OpPubArg,
    OpS,
    OpSu,
    OpSub,
    OPSubSpace,
    OpSubArg,
    OpMsgPayload,
    OpMsgEnd,
}

#[derive(Debug, PartialEq)]
pub struct SubArg<'a> {
    pub subject: &'a str,
    pub sid: &'a str,
    pub queue: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct PubArg<'a> {
    pub subject: &'a str,
    pub size_buf: &'a str, // 1024 字符串形式,避免后续再次转换
    pub size: usize,       //1024 整数形式
    pub msg: &'a [u8],
}

const BUF_LEN: usize = 512;
pub struct Parser {
    state: ParseState,
    buf: [u8; BUF_LEN],
    arg_len: usize,
    msg_buf: Option<Vec<u8>>,
    msg_total_len: usize,
    msg_len: usize,
    debug: bool,
}

#[derive(Debug, PartialEq)]
pub enum ParseResult<'a> {
    NoMsg,
    Sub(SubArg<'a>),
    Pub(PubArg<'a>),
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParseState::OpStart,
            buf: [0; BUF_LEN],
            arg_len: 0,
            msg_buf: None,
            msg_total_len: 0,
            msg_len: 0,
            debug: true,
        }
    }
    pub fn parse(&mut self, buf: &[u8]) -> Result<(ParseResult, usize), NError> {
        let mut b;
        let mut i = 0;

        if self.debug {
            println!(
                "parse string: {}, state:{:?}",
                unsafe { std::str::from_utf8_unchecked(buf) },
                self.state
            );
        }

        while i < buf.len() {
            b = buf[i] as char;
            use ParseState::*;
            match self.state {
                OpStart => match b {
                    'P' | 'p' => self.state = OpP,
                    'S' | 's' => self.state = OpS,
                    _ => parse_error!(),
                },
                OpP => match b {
                    'U' | 'u' => self.state = OpPu,
                    _ => parse_error!(),
                },
                OpPu => match b {
                    'B' | 'b' => self.state = OpPub,
                    _ => parse_error!(),
                },
                OpPub => match b {
                    ' ' | '\t' => self.state = OpPubSpace,
                    _ => parse_error!(),
                },
                OpPubSpace => match b {
                    ' ' | '\t' => {}
                    _ => {
                        self.state = OpPubArg;
                        self.arg_len = 0;
                        continue;
                    }
                },
                OpPubArg => match b {
                    '\r' => {}
                    '\n' => {
                        self.state = OpMsgPayload;
                        let size = self.process_payload_size()?;
                        if size == 0 || size > 1 * 1024 * 1024 {
                            return Err(NError::new(ERROR_MESSAGE_SIZE_TOO_LARGE));
                        }
                        if size + self.arg_len > BUF_LEN && self.msg_buf.is_none() {
                            self.msg_buf = Some(Vec::with_capacity(size));
                        }
                        self.msg_total_len = size;
                    }
                    _ => self.add_arg(b as u8)?,
                },
                OpMsgPayload => {
                    if self.msg_len < self.msg_total_len {
                        self.add_msg(b as u8);
                    } else {
                        self.state = OpMsgEnd;
                    }
                }
                OpMsgEnd => match b {
                    ' ' | '\t' => {}
                    '\n' => {
                        self.state = OpStart;
                        let res = self.process_payload()?;
                        return Ok((res, i + 1));
                    }
                    _ => parse_error!(),
                },
                OpS => match b {
                    'U' => self.state = OpSu,
                    _ => parse_error!(),
                },
                OpSu => match b {
                    'B' => self.state = OpSub,
                    _ => parse_error!(),
                },
                OpSub => match b {
                    ' ' | '\t' => self.state = OPSubSpace,
                    _ => parse_error!(),
                },
                OPSubSpace => match b {
                    ' ' | '\t' => {}
                    _ => {
                        self.state = OpSubArg;
                        self.arg_len = 0;
                        continue;
                    }
                },
                OpSubArg => match b {
                    '\r' => {}
                    '\n' => {
                        self.state = OpStart;
                        let res = self.process_sub()?;
                        return Ok((res, i + 1));
                    }
                    _ => self.add_arg(b as u8)?,
                },
            }
            i += 1;
        }
        Ok((ParseResult::NoMsg, buf.len()))
    }

    fn add_arg(&mut self, b: u8) -> Result<(), NError> {
        if self.arg_len >= self.buf.len() {
            parse_error!();
        }
        self.buf[self.arg_len] = b;
        self.arg_len += 1;
        Ok(())
    }

    fn add_msg(&mut self, b: u8) {
        if let Some(buf) = self.msg_buf.as_mut() {
            buf.push(b);
        } else {
            if self.arg_len + self.msg_total_len > BUF_LEN {
                panic!("message is large, should allocate space");
            }
            self.buf[self.arg_len + self.msg_len] = b;
        }
        self.msg_len += 1;
    }

    fn process_sub(&self) -> Result<ParseResult, NError> {
        let buf = &self.buf[0..self.arg_len];
        let s = std::str::from_utf8(buf).unwrap();
        let mut arg_buf = [""; 3];
        let mut arg_len = 0;
        for e in s.split(' ') {
            if e.len() == 0 {
                continue;
            }
            if arg_len >= 3 {
                parse_error!()
            }
            arg_buf[arg_len] = e;
            arg_len += 1;
        }
        let mut sub_arg = SubArg {
            subject: arg_buf[0],
            sid: "",
            queue: None,
        };
        match arg_len {
            2 => {
                sub_arg.sid = arg_buf[1];
            }
            3 => {
                sub_arg.sid = arg_buf[1];
                sub_arg.queue = Some(arg_buf[2]);
            }
            _ => parse_error!(),
        }
        Ok(ParseResult::Sub(sub_arg))
    }

    fn process_payload(&self) -> Result<ParseResult, NError> {
        let msg = if let Some(buf) = &self.msg_buf {
            buf.as_slice()
        } else {
            &self.buf[self.arg_len..self.arg_len + self.msg_total_len]
        };

        let s = unsafe { std::str::from_utf8_unchecked(&self.buf[0..self.arg_len]) };
        let mut arg_buf = [""; 2];
        let mut arg_len = 0;
        for e in s.split(' ') {
            if e.len() == 0 {
                continue;
            }
            arg_buf[arg_len] = e;
            arg_len += 1;
        }
        let pub_arg = PubArg {
            subject: arg_buf[0],
            size_buf: arg_buf[1],
            size: self.msg_total_len,
            msg,
        };

        Ok(ParseResult::Pub(pub_arg))
    }

    fn process_payload_size(&self) -> Result<usize, NError> {
        let buf = &self.buf[0..self.arg_len];
        let pos = buf
            .iter()
            .rev()
            .position(|b| *b == ' ' as u8 || *b == '\t' as u8);
        if pos.is_none() {
            parse_error!();
        }
        let pos = pos.unwrap();
        let size_buf = &buf[(self.arg_len - pos)..];
        let s = unsafe { std::str::from_utf8_unchecked(size_buf) };
        s.parse().map_err(|_| NError::new(ERROR_PARSE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_process_sub() {
        let mut p = Parser::new();
        let buf = "subject 5".as_bytes();
        p.buf[0..buf.len()].copy_from_slice(buf);
        p.arg_len = buf.len();
        let r = p.process_sub();
        assert!(r.is_ok());
        if let ParseResult::Sub(sub) = r.unwrap() {
            assert_eq!(sub.subject, "subject");
            assert_eq!(sub.sid, "5");
            assert!(sub.queue.is_none());
        }
    }
    #[test]
    fn test_sub() {
        let mut p = Parser::new();
        let buf = "SUB subject 1\r\n".as_bytes();
        let r = p.parse(buf);
        assert!(r.is_ok());
        println!("r={:?}", r);
        let r = r.unwrap();
        assert_eq!(r.1, buf.len());
        if let ParseResult::Sub(sub) = r.0 {
            assert_eq!(sub.subject, "subject");
            assert_eq!(sub.sid, "1");
            assert_eq!(sub.queue, None);
        } else {
            assert!(false, "unkown error");
        }

        // let buf = "SUB subject queue 1\r\n".as_bytes();
        // let r = p.parse(buf);
        // println!("r={:?}", r);
        // assert!(r.is_ok());
        // let r = r.unwrap();
        // assert_eq!(r.1, buf.len());
        // if let ParseResult::Sub(sub) = r.0 {
        //     assert_eq!(sub.subject, "subject");
        //     assert_eq!(sub.sid, "1");
        //     assert_eq!(sub.queue, Some("queue"));
        // } else {
        //     assert!(false, "unkown error");
        // }
    }

    #[test]
    fn test_process_payload_size() {
        let mut p = Parser::new();
        let buf = "FOO 11".as_bytes();
        p.buf[0..buf.len()].copy_from_slice(buf);
        p.arg_len = buf.len();
        let r = p.process_payload_size();
        assert!(r.is_ok());
        let size = r.unwrap();
        assert_eq!(size, 11);
    }

    #[test]
    fn test_process_payload() {
        let mut p = Parser::new();
        let buf = "FOO 11Hello NATS!".as_bytes();
        p.buf[0..buf.len()].copy_from_slice(buf);
        p.arg_len = "FOO 11".as_bytes().len();
        p.msg_total_len = 11;
        let r = p.process_payload();
        assert!(r.is_ok());
        if let ParseResult::Pub(pub_arg) = r.unwrap() {
            assert_eq!(pub_arg.subject, "FOO");
            assert_eq!(pub_arg.size_buf, "11");
            assert_eq!(pub_arg.size, 11);
            assert_eq!(pub_arg.msg, "Hello NATS!".as_bytes());
        }
    }

    #[test]
    fn test_pub() {
        let mut p = Parser::new();
        let buf = "PUB FOO 11\r\nHello NATS!\r\n".as_bytes();
        let r = p.parse(buf);
        assert!(r.is_ok());
        println!("r={:?}", r);
        let r = r.unwrap();
        assert_eq!(r.1, buf.len());
        if let ParseResult::Pub(pub_arg) = r.0 {
            assert_eq!(pub_arg.subject, "FOO");
            assert_eq!(pub_arg.size, 11);
            // assert_eq!(pub_arg.msg, "Hello NATS!".as_bytes());
        } else {
            assert!(false, "unkown error")
        }
    }
}
