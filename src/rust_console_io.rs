use devices::{TTY, ByteReader, ByteWriter};

pub struct RustConsoleIo {
}

impl ByteReader for RustConsoleIo {
    fn poll_nonblocking(&mut self) -> u8 {
        0                   // TODO
    }

    fn get_nonblocking(&mut self) -> u8 {
        0                       // TODO
    }
}

impl ByteWriter for RustConsoleIo {
    fn put_nonblocking(&mut self, c: u8) {
        print!("{}", char::from(c));
    }
}

impl TTY for RustConsoleIo {}

pub fn make() -> RustConsoleIo
{
    RustConsoleIo {}
}
