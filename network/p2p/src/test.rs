#[cfg(test)]
mod test {
    use super::adapter::pad_millis;
    #[test]
    fn test_pad_millis() {
        assert_eq!(pad_millis(0), Some("000"));
        assert_eq!(pad_millis(1), Some("001"));
        assert_eq!(pad_millis(10), Some("010"));
        assert_eq!(pad_millis(11), Some("011"));
        assert_eq!(pad_millis(100), Some("100"));
        assert_eq!(pad_millis(101), Some("101"));
        assert_eq!(pad_millis(110), Some("110"));
        assert_eq!(pad_millis(111), Some("111"));
        assert_eq!(pad_millis(1234), None);
    }
}
