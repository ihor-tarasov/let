use std::collections::HashMap;

pub trait Writer<W>
where
    W: std::io::Write,
{
    fn write(&self, write: &mut W) -> let_result::Result;
}

impl<W> Writer<W> for u32
where
    W: std::io::Write,
{
    fn write(&self, write: &mut W) -> let_result::Result {
        write.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl<W> Writer<W> for Box<[u8]>
where
    W: std::io::Write,
{
    fn write(&self, write: &mut W) -> let_result::Result {
        debug_assert!(self.len() <= u8::MAX as usize);
        write.write_all(&[self.len() as u8])?;
        write.write_all(&self)?;
        Ok(())
    }
}

impl<W> Writer<W> for Vec<u8>
where
    W: std::io::Write,
{
    fn write(&self, write: &mut W) -> let_result::Result {
        debug_assert!(self.len() <= u32::MAX as usize);
        write.write_all(&(self.len() as u32).to_be_bytes())?;
        write.write_all(&self)?;
        Ok(())
    }
}

impl<W> Writer<W> for Vec<u32>
where
    W: std::io::Write,
{
    fn write(&self, write: &mut W) -> let_result::Result {
        debug_assert!(self.len() <= u32::MAX as usize);
        write.write_all(&(self.len() as u32).to_be_bytes())?;
        for v in self.iter().cloned() {
            v.write(write)?;
        }
        Ok(())
    }
}

impl<W, T> Writer<W> for HashMap<Box<[u8]>, T>
where
    W: std::io::Write,
    T: Writer<W>,
{
    fn write(&self, write: &mut W) -> let_result::Result {
        debug_assert!(self.len() <= u32::MAX as usize);
        write.write_all(&(self.len() as u32).to_be_bytes())?;
        for (k, v) in self.iter() {
            k.write(write)?;
            v.write(write)?;
        }
        Ok(())
    }
}
