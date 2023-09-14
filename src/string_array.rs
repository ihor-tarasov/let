pub struct StringArray(Vec<u8>);

pub struct StringBuilder<'a> {
    array: &'a mut StringArray,
    start: usize,
}

impl<'a> StringBuilder<'a> {
    pub fn push(&mut self, c: u8) {
        self.array.0.push(c);
    }

    pub fn build(self) -> usize {
        self.array.0.push(b'\0');
        let s = &self.array.0[self.start..(self.array.0.len() - 1)];
        let index = self.array.index_of(s);
        if index != self.start {
            self.array.0.truncate(self.start);
        }
        index
    }
}

fn find_terminator(s: &[u8], mut offset: usize) -> usize {
    loop {
        if s[offset] == b'\0' {
            return offset
        } else {
            offset += 1;
        }
    }
}

impl StringArray {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self) -> StringBuilder {
        let len = self.0.len();
        StringBuilder { array: self, start: len }
    }

    pub fn index_of(&self, s: &[u8]) -> usize {
        let mut i = 0;
        loop {
            let end = find_terminator(s, i);
            if s[i..end] == self.0[i..end] {
                return i;
            } else {
                i = end;
            }
        }
    }
}

impl std::ops::Index<usize> for StringArray {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        let end = find_terminator(&self.0, index);
        &self.0[index..end]
    }
}
