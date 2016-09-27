// TODO: Need the last bits of shutdown support?

use machine::INTR_CONRDY;

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
    pub fn start(interrupt:Arc<AtomicUsize>) -> Console {
        let in_data = Arc::new(AtomicUsize::new(0));
        let out_rdy = Arc::new(AtomicBool::new(true));
        let (send_outchar, recv_outchar) = channel();

        // Output thread
        {
            let out_rdy = out_rdy.clone();
            let interrupt = interrupt.clone();
            thread::spawn(move || {
                loop {
                    match recv_outchar.recv() {
                        Ok(c) => {
                            let buf = [c];
                            stdout().write(&buf).unwrap();
                            stdout().flush().unwrap();
                            out_rdy.store(true, Ordering::SeqCst);
                            while interrupt.compare_and_swap(0, INTR_CONRDY, Ordering::SeqCst) != 0 {}
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            });
        }

        // Input thread
        {
            let in_data = in_data.clone();
            thread::spawn(move || {
                loop {
                    let mut buf = [0; 1];
                    // TODO: interrupt the blocking read if we want to halt.  Not
                    // completely clear if this is properly supported, and if so how.
                    stdin().read(&mut buf).unwrap();
                    in_data.store(DATA_AVAIL | (buf[0] as usize), Ordering::SeqCst);
                    while interrupt.compare_and_swap(0, INTR_CONRDY, Ordering::SeqCst) != 0 {}
                }
            });
        }

        return Console { in_data: in_data,
                         out_rdy: out_rdy,
                         out_chan: send_outchar };
    }

    pub fn halt(self) {
        // TODO: Should stop the threads, see comments above.
        // The output thread will exit when the main thread closes the
        // channel on its side.  But we may want to stop the input thread properly.
    }

    pub fn in_ready(&self) -> bool {
        return (self.in_data.load(Ordering::SeqCst) & DATA_AVAIL) != 0;
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
            self.out_chan.send(ch).unwrap();
        }
    }
}
