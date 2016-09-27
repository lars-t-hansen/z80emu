pub struct PIC
{
    enabled: Arc<AtomicBool>
}

impl PIC
{
    // Maybe better if this were to (also) return the data to be delivered to the devices
    // as a clonable object?  eg (PIC, device_info)

    pub fn connect(to_cpu:Arc<AtomicUsize>, to_devices:Arc<AtomicUsize>) -> PIC {
        // Fork off thread that listens on to_devices.
        // If a signal comes in:
        //  - if the PIC is enabled then
        //    - forward the signal to the cpu
        //    - hold the signal to the device high until cpu has taken it
        //    - drop the signal
        //  - Otherwise just drop the signal

        let enabled = Arc::new(AtomicBool::new(false));

        {
            let enabled = enabled.clone();
            thread::spawn(move || {
                // Hm, we don't want to busy-wait here
                // How to implement?
                // Really a limited (blocking) channel would be better for the devices,
                // or maybe a channel to deliver a ping and busy-wait on an atomic
                // until it is cleared.  Interrupt number should be delivered in variable,
                // probably.
            });
        }

        PIC { enabled: enabled }
    }

    pub fn enable(&mut self) {
        self.enabled.store(true, Ordering::Relaxed);
    }
}
