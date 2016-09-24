// TODO: Need some kind of support for interrupts.

use std::io::{Read, stdin, stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread;

pub struct Console
{
    in_data: Arc<AtomicUsize>,  // Data available in low 8 if in_data & DATA_AVAIL.
    out_rdy: Arc<AtomicBool>,   // True if out_chan can accept a character
    out_chan: Sender<u8>        // Channel for output data
}

const DATA_AVAIL: usize = 0x8000;

impl Console
{
    pub fn start() -> Console {
        let in_data = Arc::new(AtomicUsize::new(0));
        let out_rdy = Arc::new(AtomicBool::new(false));
        let (send_outchar, recv_outchar) = channel();

        // Output thread
        {
            let out_rdy = out_rdy.clone();
            thread::spawn(move || {
                loop {
                    let c = recv_outchar.recv().unwrap();
                    let buf = [c];
                    stdout().write(&buf).expect("console out");
                    stdout().flush().expect("console out");
                    out_rdy.store(true, Ordering::SeqCst);
                }
            });
        }

        // Input thread
        {
            let in_data = in_data.clone();
            thread::spawn(move || {
                loop {
                    let mut buf = [0; 1];
                    stdin().read(&mut buf).expect("console in");
                    in_data.store(DATA_AVAIL | (buf[0] as usize), Ordering::SeqCst);
                }
            });
        }

        return Console { in_data: in_data,
                         out_rdy: out_rdy,
                         out_chan: send_outchar };
    }

    pub fn halt(&self) {
        // TODO: Should stop the threads
    }

    pub fn in_ready(&self) -> bool {
        return (self.in_data.load(Ordering::SeqCst) & 0x80) != 0;
    }

    pub fn in_char(&self) -> u8 {
        let c = self.in_data.load(Ordering::SeqCst);
        self.in_data.store(0, Ordering::SeqCst);
        return c as u8;
    }

    pub fn out_ready(&self) -> bool {
        return self.out_rdy.load(Ordering::SeqCst);
    }

    pub fn out_char(&self, ch:u8) {
        if self.out_rdy.load(Ordering::SeqCst) {
            self.out_rdy.store(false, Ordering::SeqCst);
            self.out_chan.send(ch).expect("console send");
        }
    }
}
