use crate::error::*;

macro_rules! parse_error {
    () => {{
        return Err(NError::new(ERROR_PARSE));
    }};
}

#[derive(Debug, Clone)]
enum ParseState {
    OpStart,
    OpS,
    OpSu,
    OpSub,
    OPSubSpace,
    OpSubArg,
    OpP,
    OpPu,
    OpPub,
    OpPubSpace,
    OpPubArg,
    OpMsg,
    OpMsgFull,
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
                    'S' => self.state = OpS,
                    'P' => self.state = OpP,
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
                _ => panic!(),
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
            subject: "",
            sid: "",
            queue: None,
        };
        sub_arg.subject = arg_buf[0];
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
}
