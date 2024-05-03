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
    current: &[u8],
    bytes_per_pixel: usize,
) -> Vec<u8> {
    match filter {
        Filter::None => current.to_vec(),
        Filter::Sub => sub_filter(current, bytes_per_pixel),
        Filter::Up => up_filter(previous, current),
        Filter::Average => average_filter(previous, current, bytes_per_pixel),
        Filter::Paeth => paeth_filter(previous, current, bytes_per_pixel),
    }
}

fn sub_filter(current: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let mut result = vec![];
    for i in 0..current.len() {
        let a = if i < bytes_per_pixel {
            0
        } else {
            result[i - bytes_per_pixel]
        };
        result.push(current[i].wrapping_add(a));
    }
    result
}

fn up_filter(previous: &[u8], current: &[u8]) -> Vec<u8> {
    current
        .iter()
        .enumerate()
        .map(|(i, &x)| x.wrapping_add(previous[i]))
        .collect()
}

fn average_filter(previous: &[u8], current: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let mut result = vec![];
    for i in 0..current.len() {
        let a = if i < bytes_per_pixel {
            0
        } else {
            result[i - bytes_per_pixel]
        };
        let b = previous.get(i).copied().unwrap_or(0);
        result.push(current[i].wrapping_add(((a as u16 + b as u16) / 2) as u8));
    }
    result
}

fn paeth_filter(previous: &[u8], current: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let mut result = vec![];
    for i in 0..current.len() {
        let a = if i < bytes_per_pixel {
            0
        } else {
            current[i - bytes_per_pixel]
        };
        let b = previous.get(i).copied().unwrap_or(0);
        let c_idx = if (i as i16 - bytes_per_pixel as i16) < 0 {
            0
        } else {
            i - bytes_per_pixel
        };

        let c = previous.get(c_idx).copied().unwrap_or(0);
        result.push(current[i].wrapping_add(paeth_predictor(a, b, c)));
    }
    result
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;
    let p_a = (p - a as i32).abs();
    let p_b = (p - b as i32).abs();
    let p_c = (p - c as i32).abs();
    if p_a <= p_b && p_a <= p_c {
        a
    } else if p_b <= p_c {
        b
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_filter() {
        let current = vec![0x02, 0x03, 0x04];
        let bytes_per_pixel = 1;
        let expected = vec![0x02, 0x05, 0x09];

        let result = sub_filter(&current, bytes_per_pixel);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_up_filter() {
        let previous = vec![0x02, 0x03, 0x04];
        let current = vec![0x03, 0x04, 0x05];
        let expected = vec![0x05, 0x07, 0x09];

        let result = up_filter(&previous, &current);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_average_filter() {
        let previous = vec![0x02, 0x03, 0x04];
        let current = vec![0x03, 0x04, 0x05];
        let bytes_per_pixel = 1;
        let expected = vec![0x04, 0x07, 0x0a];

        let result = average_filter(&previous, &current, bytes_per_pixel);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_paeth_filter() {
        let previous = vec![0x02, 0x03, 0x04];
        let current = vec![0x03, 0x04, 0x05];
        let bytes_per_pixel = 1;
        let expected = vec![0x03, 0x07, 0x09];

        let result = paeth_filter(&previous, &current, bytes_per_pixel);

        assert_eq!(result, expected);
    }
}
