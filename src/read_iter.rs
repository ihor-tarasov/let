fn try_to_fill_the_buffer<R>(read: &mut R, buf: &mut [u8]) -> std::io::Result<usize>
where
    R: std::io::Read,
{
    let mut count = 0;
    while count != buf.len() {
        match read.read(&mut buf[count..]) {
            Ok(0) => break,
            Ok(n) => {
                count += n;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }

    Ok(count)
}

pub struct ReadIter<R> {
    read: R,
    buffer: Box<[u8]>,
    buffer_size: usize,
    offset: usize,
    error: Option<std::io::Error>,
}

impl<R> ReadIter<R> {
    pub fn new(read: R, buffer_size: usize) -> Self {
        Self {
            read,
            buffer: vec![0; buffer_size].into_boxed_slice(),
            buffer_size: 0,
            offset: 0,
            error: None,
        }
    }

    pub fn get_error(&self) -> Option<&std::io::Error> {
        self.error.as_ref()
    }
}

impl<R> Iterator for ReadIter<R>
where
    R: std::io::Read,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.buffer_size {
            self.offset += 1;
            Some(self.buffer[self.offset - 1])
        } else {
            match try_to_fill_the_buffer(&mut self.read, &mut self.buffer) {
                Ok(0) => None,
                Ok(size) => {
                    self.offset -= self.buffer_size;
                    self.buffer_size = size;
                    self.buffer.first().cloned()
                }
                Err(error) => {
                    self.error = Some(error);
                    None
                }
            }
        }
    }
}
