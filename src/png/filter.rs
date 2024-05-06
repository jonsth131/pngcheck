#[derive(Debug)]
pub enum Filter {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

pub fn filter_scanline(
    filter: Filter,
    previous: &[u8],
    current: &mut [u8],
    bytes_per_pixel: usize,
) {
    match filter {
        Filter::None => (),
        Filter::Sub => sub_filter(current, bytes_per_pixel),
        Filter::Up => up_filter(previous, current),
        Filter::Average => average_filter(previous, current, bytes_per_pixel),
        Filter::Paeth => paeth_filter(previous, current, bytes_per_pixel),
    }
}

fn sub_filter(current: &mut [u8], bytes_per_pixel: usize) {
    let mut a = vec![0; bytes_per_pixel];
    for x in current.chunks_exact_mut(bytes_per_pixel) {
        for (x, a) in x.iter_mut().zip(a.iter()) {
            *x = x.wrapping_add(*a);
        }
        a = x.to_vec();
    }
}

fn up_filter(previous: &[u8], current: &mut [u8]) {
    current.iter_mut().zip(previous.iter()).for_each(|(x, a)| {
        *x = x.wrapping_add(*a);
    });
}

fn average_filter(previous: &[u8], current: &mut [u8], bytes_per_pixel: usize) {
    let mut a = vec![0; bytes_per_pixel];
    for (x, b) in current
        .chunks_exact_mut(bytes_per_pixel)
        .zip(previous.chunks_exact(bytes_per_pixel))
    {
        for ((x, a), b) in x.iter_mut().zip(a.iter()).zip(b.iter()) {
            *x = x.wrapping_add(((*a as i16 + *b as i16) / 2) as u8);
        }
        a = x.to_vec();
    }
}

fn paeth_filter(previous: &[u8], current: &mut [u8], bytes_per_pixel: usize) {
    let mut a = vec![0; bytes_per_pixel];
    let mut c = vec![0; bytes_per_pixel];

    for (x, b) in current
        .chunks_exact_mut(bytes_per_pixel)
        .zip(previous.chunks_exact(bytes_per_pixel))
    {
        for (((x, a), b), c) in x.iter_mut().zip(a.iter()).zip(b.iter()).zip(c.iter()) {
            let p = *a as i16 + *b as i16 - *c as i16;
            let pa = (p - *a as i16).abs();
            let pb = (p - *b as i16).abs();
            let pc = (p - *c as i16).abs();
            *x = x.wrapping_add(if pa <= pb && pa <= pc {
                *a
            } else if pb <= pc {
                *b
            } else {
                *c
            });
        }
        a = x.to_vec();
        c = b.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_filter() {
        let mut current = vec![0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let expected = vec![2, 3, 4, 5, 8, 10, 12, 14];

        sub_filter(&mut current, 4);

        assert_eq!(current, expected);
    }

    #[test]
    fn test_up_filter() {
        let previous = vec![0x02, 0x03, 0x04];
        let mut current = vec![0x03, 0x04, 0x05];
        let expected = vec![0x05, 0x07, 0x09];

        up_filter(&previous, &mut current);

        assert_eq!(current, expected);
    }

    #[test]
    fn test_average_filter() {
        let previous = vec![0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut current = vec![0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
        let expected = vec![4, 5, 7, 8, 12, 14, 16, 18];

        average_filter(&previous, &mut current, 4);

        assert_eq!(current, expected);
    }

    #[test]
    fn test_paeth_filter() {
        let previous = vec![0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut current = vec![0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
        let expected = vec![5, 7, 9, 11, 13, 15, 18, 21];

        paeth_filter(&previous, &mut current, 4);

        assert_eq!(current, expected);
    }
}
