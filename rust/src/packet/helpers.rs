pub(crate) fn normalize_timestamp(raw: i64, baseline: Option<i64>) -> i64 {
    baseline.map(|base| raw.saturating_sub(base)).unwrap_or(raw)
}

#[cfg(test)]
mod tests {
    use super::normalize_timestamp;

    #[test]
    fn normalize_timestamp_with_baseline() {
        assert_eq!(normalize_timestamp(2_000, Some(1_500)), 500);
        assert_eq!(normalize_timestamp(2_000, None), 2_000);
    }
}
