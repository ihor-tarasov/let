use std::collections::HashMap;

pub trait Reader<R, T>
where
    R: std::io::Read,
{
    fn read(read: &mut R) -> let_result::Result<T>;
}

impl<R> Reader<R, u32> for u32
where
    R: std::io::Read,
{
    fn read(read: &mut R) -> let_result::Result<u32> {
        let mut buf = [0u8; std::mem::size_of::<u32>()];
        read.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl<R> Reader<R, Box<[u8]>> for Box<[u8]>
where
    R: std::io::Read,
{
    fn read(read: &mut R) -> let_result::Result<Box<[u8]>> {
        let mut len_buf = [0u8; 1];
        read.read_exact(&mut len_buf)?;
        let mut buf = vec![0u8; len_buf[0] as usize].into_boxed_slice();
        read.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<R> Reader<R, Vec<u8>> for Vec<u8>
where
    R: std::io::Read,
{
    fn read(read: &mut R) -> let_result::Result<Vec<u8>> {
        let mut len_buf = [0u8; std::mem::size_of::<u32>()];
        read.read_exact(&mut len_buf)?;
        let mut buf = vec![0u8; u32::from_be_bytes(len_buf) as usize];
        read.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<R> Reader<R, Vec<u32>> for Vec<u32>
where
    R: std::io::Read,
{
    fn read(read: &mut R) -> let_result::Result<Vec<u32>> {
        let mut len_buf = [0u8; std::mem::size_of::<u32>()];
        read.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf);
        let mut result = Vec::with_capacity(len as usize);
        for _ in 0..len {
            result.push(u32::read(read)?);
        }
        Ok(result)
    }
}

impl<R, T> Reader<R, HashMap<Box<[u8]>, T>> for HashMap<Box<[u8]>, T>
where
    R: std::io::Read,
    T: Reader<R, T>,
{
    fn read(read: &mut R) -> let_result::Result<HashMap<Box<[u8]>, T>> {
        let mut len_buf = [0u8; std::mem::size_of::<u32>()];
        read.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf);
        let mut result = HashMap::with_capacity(len as usize);
        for _ in 0..len {
            let key = Box::<[u8]>::read(read)?;
            if result.contains_key(&key) {
                return let_result::raise!("labels conflict.");
            }
            let value = T::read(read)?;            
            result.insert(key, value);
        }
        Ok(result)
    }
}
