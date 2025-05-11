use std::iter::Iterator;

#[derive(Debug)]
pub struct Bitmap1D {
    pub len: usize,
    pub data: Vec<u64>
}

impl Bitmap1D {
    pub fn zeros(len: usize) -> Self {
        let datasize = (len + 63) / 64;
        Self { len, data: vec![0; datasize] }
    }

    pub fn mask_oob(&mut self) {
        let last_chunk_id = (self.len + 63) / 64;
        if let Some(chunk) = self.data.get_mut(last_chunk_id-1) {
            let bitmask: u64 = !(!0 >> (self.len%64));
            *chunk &= bitmask;
        }
        for item in self.data[last_chunk_id..].iter_mut() {
            *item = 0;
        }
    }

    pub fn sub_bitmap(&self, start_bit: usize, last_bit: Option<usize>) -> Self {
        let last_bit = last_bit.unwrap_or(self.len - 1);
        let start_chunk = start_bit >> 6; // / 64
        let last_chunk = last_bit >> 6;
        let new_arr = &self.data[start_chunk..(last_chunk+1)];
        let mut new_data: Vec<u64> = Vec::with_capacity(new_arr.len());

        let bits_per_chunk = start_bit % 64;

        for (i, &item) in new_arr.iter().enumerate() {
            let mut to_add = item << bits_per_chunk;
            if let Some(next_item) = new_arr.get(i+1) {
                to_add += next_item >> (64 - bits_per_chunk); //TODO check ?
            }
            new_data.push(to_add);
        }
        Self { len: last_bit - start_bit + 1, data: new_data }
    }

    pub fn pad(&self, pad_left: usize, pad_right: usize) -> Self {
        let new_len = self.len + pad_left + pad_right;
        let new_data_len = (new_len + 63) >> 6;
        let mut new_data: Vec<u64> = Vec::with_capacity(new_data_len);

        let chunk_pad = pad_left >> 6;
        let bits_per_chunk = pad_left % 64;

        for chunk_id in 0..new_data_len {
            let mut new_chunk: u64 = 0;
            let chunk2_id = chunk_id.checked_sub(chunk_pad);
            let chunk1_id = chunk2_id.and_then(|x| x.checked_sub(1));
            if let Some(chunk1) = chunk1_id.and_then(|c| self.data.get(c)) {
                //new_chunk += chunk1 << (64 - bits_per_chunk);
                new_chunk += chunk1.checked_shl((64 - bits_per_chunk) as u32).unwrap_or(0);
            }
            if let Some(chunk2) = chunk2_id.and_then(|c| self.data.get(c)) {
                new_chunk += chunk2 >> bits_per_chunk;
            }
            new_data.push(new_chunk);
        }
        Self { len: new_len, data: new_data }
    }

    //pub fn get(&self, coord: usize) -> Option<bool> {
    //    let chunk = self.data.get(coord / 64)?;
    //    return Some((chunk >> (coord % 64)) % 2 == 1);
    //}

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        let mut remaining_bits = self.len;
        for item in &self.data {
            let mut binary = &format!("{item:064b}")[..];
            if remaining_bits < 64 {
                binary = &binary[..remaining_bits];
            }
            out.push_str(binary);
            remaining_bits = match remaining_bits.checked_sub(64) {
                Some(i) => i,
                None => break
            };
        }
        out
    }
}

pub struct Bitmap2D {
    pub shape: (usize, usize),
    pub data: Vec<u64>
}

impl Bitmap2D {
    pub fn get_lines(&self) -> Vec<Bitmap1D> {
        let mut out = Vec::with_capacity(self.shape.0);
        let line = Bitmap1D { len: self.shape.0*self.shape.1,
                              data: self.data.clone()
                            };
        for line_id in 0..self.shape.0 {
            out.push(line.sub_bitmap(line_id*self.shape.1, Some((line_id+1)*self.shape.1 - 1)));
        }
        return out
    }

    pub fn to_string(&self) -> String {
        let lines = self.get_lines();
        let lines_str: Vec<_> = lines.into_iter().map(|l| l.to_string()).collect();
        return lines_str.join("\n");
    }

    pub fn stack(mut lines: Vec<Bitmap1D>) -> Self {
        let first_line = match lines.get(0) {
            Some(l) => l,
            None => return Bitmap2D { shape: (0,0), data: Vec::new() }
        };
        let line_len = first_line.len;
        let new_len = line_len*lines.len();
        let mut new_arr = vec![0; (new_len + 63)/64];
        for (i, line) in lines.iter_mut().enumerate() {
            line.mask_oob();
            let padded = line.pad(i*line_len, new_len - (i+1)*line_len);
            for (item, to_add) in new_arr.iter_mut().zip(padded.data) {
                *item += to_add;
            }
        }
        Bitmap2D { shape: (lines.len(), line_len), data: new_arr }
    }

    pub fn pad_to(&self, target_shape: (usize, usize), offset: (usize, usize)) -> Self {
        let mut lines = self.get_lines();
        for line in lines.iter_mut() { line.mask_oob() }
        let padded_lines = lines.iter()
            .map(|line| line.pad(offset.1, target_shape.1 - self.shape.1 - offset.1));
        let mut to_stack: Vec<Bitmap1D> = Vec::with_capacity(target_shape.0);
        for _ in 0..offset.0 { to_stack.push(Bitmap1D::zeros(target_shape.1)) }
        for line_pad in padded_lines { to_stack.push(line_pad); }
        for _ in (offset.0 + self.shape.0)..target_shape.0 {
            to_stack.push(Bitmap1D::zeros(target_shape.1));
        }
        Self::stack(to_stack)
    }

    pub fn get(&self, coord: (usize, usize)) -> Option<bool> {
        let idx = coord.0*self.shape.1 + coord.1;
        let chunk = self.data.get(idx / 64)?;
        return Some((chunk >> (63 - (idx % 64))) % 2 == 1);
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.data.iter().zip(other.data.iter()).map(|(x,y)| x & y).sum::<u64>() > 0
    }

    pub fn or(&self, other: &Self) -> Self {
        let newdata: Vec<u64> =
            self.data.iter()
                .zip(other.data.iter())
                .map(|(x,y)| x | y).collect();
        Bitmap2D { shape : self.shape, data: newdata }
    }

    pub fn print_all<'a>(mut bitmap_iter: impl Iterator<Item=&'a Self>) -> String {
        let mut char_vec = match bitmap_iter.next() {
            Some(bitmap) => bitmap.to_string().chars().collect::<Vec<char>>(),
            None => return String::new()
        };
            
        for (idx, bitmap) in bitmap_iter.enumerate() {
            char_vec = bitmap.to_string()
                .replace('1', &format!("{:0x}", ((idx+2)%16))[..])
                .chars().zip(char_vec)
                .map(|(new, old)| if new != '0' { new } else { old })
                .collect::<Vec<char>>();
        }
        char_vec.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_map() {
        let bm = Bitmap1D { len: 9, data: vec![0b110110101 << (64-9)] };
        assert_eq!(bm.to_string(), "110110101");
    }

    #[test]
    fn sub_bitmap() {
        let bm = Bitmap1D { len: 64, data: vec![0b1101101101101] }.sub_bitmap(64-13, None);
        assert_eq!(bm.to_string(), "1101101101101");
        let bm = Bitmap1D { len: 100, data: vec![0b1101, 0b1011 << 60] }.sub_bitmap(60, Some(67));
        assert_eq!(bm.to_string(), "11011011");
        let bm = Bitmap1D { len: 100, data: vec![0b1101, 0b1011 << 60] }.sub_bitmap(60, Some(63));
        assert_eq!(bm.to_string(), "1101");
        let bm = Bitmap1D { len: 100, data: vec![0b1101, 0b1011 << 60] }.sub_bitmap(64, Some(67));
        assert_eq!(bm.to_string(), "1011");
    }

    #[test]
    fn print_2d() {
        let bm = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
        assert_eq!(bm.to_string(), "01\n01\n11");
        let bm = Bitmap2D { shape: (2, 3), data: vec![0b010111 << 58] };
        assert_eq!(bm.to_string(), "010\n111");
    }

    #[test]
    fn pad_1d() {
        let bm = Bitmap1D { len: 5, data: vec![0b11011 << (64-5)] };
        assert_eq!(bm.pad(0, 0).to_string(), bm.to_string());
        let padded = bm.pad(3, 2);
        assert_eq!(padded.to_string(), "0001101100");
        let padded = bm.pad(61, 12);
        assert_eq!(padded.len, 61+5+12);
        assert_eq!(padded.sub_bitmap(61, Some(65)).to_string(), "11011");

        let bm = Bitmap1D { len: 112, data: vec![0xabcd0000dcba0000, 0xffbb0000aacc0000] };
        assert_eq!(bm.pad(16, 0).sub_bitmap(16, Some(127)).to_string(),
                   bm.to_string());
        assert_eq!(bm.pad(18, 0).sub_bitmap(18, Some(129)).to_string(),
                   bm.to_string());
    }

    #[test]
    fn mask() {
        let mut bm = Bitmap1D { len: 3, data: vec![0b101111 << 58] };
        let padded = bm.pad(0, 3);
        assert_eq!(padded.to_string(), "101111");
        bm.mask_oob();
        let padded = bm.pad(0, 3);
        assert_eq!(padded.to_string(), "101000");
    }

    #[test]
    fn stack_lines() {
        let bm = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
        let lines = bm.get_lines();
        let stacked = Bitmap2D::stack(lines);
        assert_eq!(bm.to_string(), stacked.to_string());
    }

    #[test]
    fn pad() {
        let j_piece = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
        let padded = j_piece.pad_to((4, 4), (0,0));
        assert_eq!(padded.to_string(), "0100\n0100\n1100\n0000");
        let padded = j_piece.pad_to((4, 4), (1,2));
        assert_eq!(padded.to_string(), "0000\n0001\n0001\n0011");
        let padded = j_piece.pad_to((7, 10), (2, 3));
        assert_eq!(padded.to_string(),
                   "0000000000\n0000000000\n0000100000\n0000100000\n0001100000\n0000000000\n0000000000");
    }

    #[test]
    fn get() {
        let j_piece = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
        assert_eq!(j_piece.get((1,1)), Some(true));
        assert_eq!(j_piece.get((1,0)), Some(false));
        let padded = j_piece.pad_to((7, 10), (2, 3));
        assert_eq!(padded.get((3,4)), Some(true));
        assert_eq!(j_piece.get((3,3)), Some(false));
    }

    #[test]
    fn intersection() {
        let j_piece = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
        let padded = j_piece.pad_to((4, 4), (0,0));
        let padded11 = j_piece.pad_to((4, 4), (1,1));
        let padded01 = j_piece.pad_to((4, 4), (0,1));
        assert!(padded.intersects(&padded));
        assert!(!padded.intersects(&padded11));
        assert!(padded.intersects(&padded01));
        assert!(padded11.intersects(&padded01));

    }

    #[test]
    fn print_bitmap_iter() {
        let j_piece = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
        let padded1 = j_piece.pad_to((4, 4), (0,0));
        //assert_eq!(padded1.to_string(), "0100\n0100\n1100\n0000");
        let padded2 = j_piece.pad_to((4, 4), (1,2));
        //assert_eq!(padded1.to_string(), "0000\n0001\n0001\n0011");
        let added = Bitmap2D::print_all(vec![padded1, padded2].iter());
        assert_eq!(added, "0100\n0102\n1102\n0022");
    }
    
}

