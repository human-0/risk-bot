#[allow(unused_imports)]
use std::{
    fs,
    io::{self, Read, Write},
};

use risk_shared::{player::PlayerId, query::Query, record::Move, serde::SerializeMove};

const MAX_CHARACTERS_READ: usize = 1000000;

#[cfg(not(target_os = "wasi"))]
pub struct Connection {
    to_engine_pipe: fs::File,
    from_engine_pipe: fs::File,
}

#[cfg(target_os = "wasi")]
pub struct Connection {}

#[cfg(not(target_os = "wasi"))]
impl Connection {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            to_engine_pipe: fs::OpenOptions::new()
                .write(true)
                .open("./io/to_engine.pipe")?,
            from_engine_pipe: fs::OpenOptions::new()
                .read(true)
                .open("./io/from_engine.pipe")?,
        })
    }

    fn receive(&mut self) -> io::Result<String> {
        let mut buffer = Vec::new();
        while buffer.len() < MAX_CHARACTERS_READ.ilog10() as usize + 1
            && buffer.last() != Some(&b',')
        {
            buffer.push(0);

            let len = buffer.len();
            self.from_engine_pipe
                .read_exact(&mut buffer[len - 1..len])?;
        }

        if buffer.pop() != Some(b',') {
            println!("{:?}", buffer);
            panic!();
        }
        assert!(buffer.len() < MAX_CHARACTERS_READ);

        let size = std::str::from_utf8(&buffer).unwrap().parse().unwrap();

        buffer.clear();
        buffer.resize(size, 0);
        self.from_engine_pipe.read_exact(&mut buffer)?;

        Ok(String::from_utf8(buffer).unwrap())
    }

    fn send(&mut self, data: &str) -> io::Result<()> {
        write!(self.to_engine_pipe, "{},{}", data.len(), data)?;
        self.to_engine_pipe.flush()
    }
}

#[cfg(target_os = "wasi")]
impl Connection {
    pub fn new() -> io::Result<Self> {
        Ok(Self {})
    }

    fn receive(&mut self) -> io::Result<String> {
        unsafe {
            let buffer = std::ptr::addr_of_mut!(BUFFER);
            let len = read_pipe(buffer.cast::<u8>()) as usize;

            let mut data = Vec::with_capacity(len);
            std::ptr::copy_nonoverlapping(buffer.cast::<u8>(), data.as_mut_ptr(), len);
            data.set_len(len);
            Ok(String::from_utf8(data).unwrap())
        }
    }

    fn send(&mut self, data: &str) -> io::Result<()> {
        unsafe {
            write_pipe(data.as_ptr(), data.len() as i32);
        }

        Ok(())
    }
}

impl Connection {
    pub fn get_next_query(&mut self) -> Query {
        serde_json::from_str(&self.receive().unwrap()).unwrap()
    }

    pub fn send_move(&mut self, player: PlayerId, mov: Move) {
        self.send(&serde_json::to_string(&SerializeMove(player, mov)).unwrap())
            .unwrap()
    }
}

#[cfg(target_os = "wasi")]
const BUFFER_SIZE: usize = MAX_CHARACTERS_READ + MAX_CHARACTERS_READ.ilog10() as usize + 1;

#[cfg(target_os = "wasi")]
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

#[cfg(target_os = "wasi")]
extern "C" {
    fn read_pipe(ptr: *mut u8) -> i32;

    fn write_pipe(ptr: *const u8, len: i32);
}
