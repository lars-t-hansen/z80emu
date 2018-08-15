// Device traits.

///////////////////////////////////////////////////////////////////////////////
//
// Serial devices provide byte-by-byte input and output.

pub trait ByteReader
{
    fn poll_nonblocking(&mut self) -> u8; // 0xFF if byte is ready, 0x00 if not
    fn get_nonblocking(&mut self) -> u8;  // Byte value
}

pub trait ByteWriter
{
    fn put_nonblocking(&mut self, c: u8);
}


///////////////////////////////////////////////////////////////////////////////
//
// A teletype has a serial typewriter and a serial keyboard but is otherwise not
// very interesting.

pub trait TTY : ByteReader + ByteWriter {}


///////////////////////////////////////////////////////////////////////////////
//
// Spinning disks have heads, tracks (cylinders), and sectors, and separate the
// seek operation from read and write operations.
//
// For simplicity's sake all our spinning disks have DMA.
//
// For simplicity's sake we have common status values for all spinning disks.

pub trait SpinningDisk
{
    // Get the disk status.  The status is set by disk_operation().
    fn get_status(&mut self) -> SpinningDiskStatus;

    // Set disk parameters.  These take effect at the next seek().
    fn set_head(&mut self, n: u8);
    fn set_track(&mut self, n: u8);
    fn set_sector(&mut self, n: u8);

    // Set the data transfer address.
    fn set_dma_high(&mut self, n: u8);
    fn set_dma_low(&mut self, n: u8);

    // The command set is specific to the particular device implementation.
    // Generally speaking, these are typical operations:
    //
    // CLEAR.  Set the status to Ready but do not invalidate the disk parameters
    // that have been set.
    //
    // SEEK.  If the status is not Ready this does nothing.  Validate disk
    // parameters and seek to sector.  Sets the status to Done once the the
    // parameters are valid and the seek has succeeded, otherwise to SeekError.
    //
    // READ.  If the status is not Ready this does nothing.  Otherwise, read the
    // selected (seeked-to) sector storing the bytes in memory at the selected
    // DMA address (with wraparound).  Sets status to ReadError if the read
    // fails; once the read is completed the status becomes Done.
    //
    // WRITE.  If the status is not Ready this does nothing.  Otherwise, write the
    // selected (seeked-to) sector from the bytes in memory at the selected DMA
    // address (with wraparound).  Sets status to WriteError if the write fails;
    // once the write is completed the status becomes Done.
    //
    // If the operation is not known or is issued when the device is not Ready
    // then status is set to OpError.
    fn disk_operation(&mut self, op: u8, mem: &mut [u8]);
}

#[derive(PartialEq,Clone,Copy)]
pub enum SpinningDiskStatus {
    Ready = 0x00,
    Done = 0x01,
    OpError = 0xFF,
    SeekError = 0xFE,
    ReadError = 0xFD,
    WriteError = 0xFC
}
