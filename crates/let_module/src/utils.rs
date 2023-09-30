pub fn write_u8<W: std::io::Write>(write: &mut W, data: u8) -> let_result::Result {
    write.write_all(&[data])?;
    Ok(())
}

pub fn write_u32<W: std::io::Write>(write: &mut W, data: u32) -> let_result::Result {
    write.write_all(&data.to_be_bytes())?;
    Ok(())
}

pub fn write_label<W: std::io::Write>(write: &mut W, data: &[u8]) -> let_result::Result {
    debug_assert!(data.len() <= u8::MAX as usize);
    write_u8(write, data.len() as u8)?;
    write.write_all(&data)?;
    Ok(())
}

pub fn write_labels<W: std::io::Write>(
    write: &mut W,
    first: &[u8],
    second: &[u8],
) -> let_result::Result {
    debug_assert!(first.len() + second.len() + 1 <= u8::MAX as usize);
    write_u8(write, (first.len() + second.len() + 1) as u8)?;
    write.write_all(&first)?;
    write_u8(write, b'.')?;
    write.write_all(&second)?;
    Ok(())
}

pub fn write_u8_slice<W: std::io::Write>(write: &mut W, data: &[u8]) -> let_result::Result {
    debug_assert!(data.len() <= u32::MAX as usize);
    write_u32(write, data.len() as u32)?;
    for value in data {
        write_u8(write, *value)?;
    }
    Ok(())
}

pub fn write_u32_slice<W: std::io::Write>(write: &mut W, data: &[u32]) -> let_result::Result {
    debug_assert!(data.len() <= u32::MAX as usize);
    write_u32(write, data.len() as u32)?;
    for value in data {
        write_u32(write, *value)?;
    }
    Ok(())
}

pub fn read_u8<R: std::io::Read>(read: &mut R) -> let_result::Result<u8> {
    let mut buf = [0];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_u32<R: std::io::Read>(read: &mut R) -> let_result::Result<u32> {
    let mut buf = [0, 0, 0, 0];
    read.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub fn read_label<R: std::io::Read>(read: &mut R) -> let_result::Result<Box<[u8]>> {
    let len = read_u8(read)?;
    let mut buf = vec![0u8; len as usize].into_boxed_slice();
    read.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn read_u8_vec<R: std::io::Read>(read: &mut R) -> let_result::Result<Vec<u8>> {
    let len = read_u32(read)?;
    let mut buf = vec![0u8; len as usize];
    read.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn read_u32_vec<R: std::io::Read>(read: &mut R) -> let_result::Result<Vec<u32>> {
    let len = read_u32(read)?;
    let mut result = Vec::with_capacity(len as usize);
    for _ in 0..len {
        result.push(read_u32(read)?);
    }
    Ok(result)
}
