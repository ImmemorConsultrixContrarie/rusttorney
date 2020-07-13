const CRYPT_CONST_1: u32 = 53761;
const CRYPT_CONST_2: u32 = 32618;
const CRYPT_KEY: u32 = 5;

fn fanta_encrypt(data: &str) -> String {
    data.chars().fold((String::with_capacity(data.len() / 2), CRYPT_KEY), |(mut s, key), c| {
        let val = (c as u32)^((key & 0xffff) >> 8);
        s.push_str(&format!("{:X}", val));
        (s, ((val + key).wrapping_mul(CRYPT_CONST_1)).wrapping_add(CRYPT_CONST_2))
    }).0
}

fn fanta_decrypt(data: &str) -> String {
    let mut ret = String::with_capacity(data.len() * 2);
    let mut key = CRYPT_KEY;
    // will be a valid hex str
    let data_bytes = hex::decode(data).unwrap();
    for byte in data_bytes {
        let val = byte as u32 ^ ((key & 0xffff) >> 8);
        ret.push(val as u8 as char);
        key = ((byte as u32 + key) * CRYPT_CONST_1) + CRYPT_CONST_2
    };
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fanta_decrypt_encrypt() {
        let data = "MS";

        assert_eq!("4D90", fanta_encrypt(data))
    }
}