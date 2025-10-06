#![cfg(test)]
use crate::core::buildings::filters::content_filter::ContentFilter;
use crate::core::buildings::filters::ip_filter::IpFilter;
use crate::core::buildings::filters::length_filter::LengthFilter;
use crate::core::buildings::filters::port_filter::PortFilter;
use crate::core::packet::{Packet, Protocol};

fn create_test_packet() -> Packet {
    Packet::new(
        "192.168.1.10".to_string(),
        "8.8.8.8".to_string(),
        12345,
        80,
        Protocol::Tcp,
        256,
        b"some payload data".to_vec(),
    )
}

#[test]
fn test_ip_filter_source_direction() {
    use crate::core::buildings::filters::ip_filter::{IpFilterConfig, IpFilterDirection};

    let config = IpFilterConfig {
        target_ip: "192.168.1.10".to_string(),
        direction: IpFilterDirection::Source,
    };
    let filter = IpFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // source_ip = "192.168.1.10"
    let mut packet_no_match = create_test_packet();
    packet_no_match.source_ip = "10.0.0.5".to_string();

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_ip_filter_destination_direction() {
    use crate::core::buildings::filters::ip_filter::{IpFilterConfig, IpFilterDirection};

    let config = IpFilterConfig {
        target_ip: "8.8.8.8".to_string(),
        direction: IpFilterDirection::Destination,
    };
    let filter = IpFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // dest_ip = "8.8.8.8"
    let mut packet_no_match = create_test_packet();
    packet_no_match.dest_ip = "10.0.0.5".to_string();

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_ip_filter_no_config() {
    let filter = IpFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 設定がない場合は何も通さない
    assert!(!filter.filter(&packet));
}

#[test]
fn test_ip_filter_set_config() {
    use crate::core::buildings::filters::ip_filter::{IpFilterConfig, IpFilterDirection};

    let mut filter = IpFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 初期状態では何も通さない
    assert!(!filter.filter(&packet));

    // 設定後はフィルタリングが有効になる
    let config = IpFilterConfig {
        target_ip: "192.168.1.10".to_string(),
        direction: IpFilterDirection::Source,
    };
    filter.set_config(config);
    assert!(filter.filter(&packet));
}

#[test]
fn test_length_filter_exact_direction() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let config = LengthFilterConfig {
        threshold: 256,
        direction: LengthFilterDirection::Exact,
    };
    let filter = LengthFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // length = 256
    let mut packet_no_match = create_test_packet();
    packet_no_match.length = 128;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_length_filter_less_than_direction() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let config = LengthFilterConfig {
        threshold: 300,
        direction: LengthFilterDirection::LessThan,
    };
    let filter = LengthFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // length = 256
    let mut packet_no_match = create_test_packet();
    packet_no_match.length = 300;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_length_filter_greater_than_direction() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let config = LengthFilterConfig {
        threshold: 200,
        direction: LengthFilterDirection::GreaterThan,
    };
    let filter = LengthFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // length = 256
    let mut packet_no_match = create_test_packet();
    packet_no_match.length = 200;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_length_filter_no_config() {
    let filter = LengthFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 設定がない場合は何も通さない
    assert!(!filter.filter(&packet));
}

#[test]
fn test_length_filter_set_config() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let mut filter = LengthFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 初期状態では何も通さない
    assert!(!filter.filter(&packet));

    // 設定後はフィルタリングが有効になる
    let config = LengthFilterConfig {
        threshold: 256,
        direction: LengthFilterDirection::Exact,
    };
    filter.set_config(config);
    assert!(filter.filter(&packet));
}

#[test]
fn test_content_filter_valid_pattern() {
    use crate::core::buildings::filters::content_filter::ContentFilterConfig;

    let config = ContentFilterConfig {
        pattern: "payload".to_string(),
    };
    let filter = ContentFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // payload contains "payload"
    let mut packet_no_match = create_test_packet();
    packet_no_match.payload = b"different data".to_vec();

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_length_filter_config_pattern() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let config = LengthFilterConfig {
        threshold: 256,
        direction: LengthFilterDirection::Exact,
    };
    let filter = LengthFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // length = 256
    let mut packet_no_match = create_test_packet();
    packet_no_match.length = 128;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_length_filter_direction_greater_than() {
    use crate::core::buildings::filters::length_filter::{
        LengthFilterConfig, LengthFilterDirection,
    };

    let config = LengthFilterConfig {
        threshold: 128,
        direction: LengthFilterDirection::GreaterThan,
    };
    let filter = LengthFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // length = 256 > 128
    let mut packet_no_match = create_test_packet();
    packet_no_match.length = 64; // 64 < 128

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_content_filter_regex_pattern() {
    use crate::core::buildings::filters::content_filter::ContentFilterConfig;

    let config = ContentFilterConfig {
        pattern: r"p.*load".to_string(), // regex pattern
    };
    let filter = ContentFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // payload matches p.*load
    let mut packet_no_match = create_test_packet();
    packet_no_match.payload = b"different data".to_vec();

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_content_filter_no_config() {
    let filter = ContentFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 設定がない場合は何も通さない
    assert!(!filter.filter(&packet));
}

#[test]
fn test_content_filter_set_config() {
    use crate::core::buildings::filters::content_filter::ContentFilterConfig;

    let mut filter = ContentFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 初期状態では何も通さない
    assert!(!filter.filter(&packet));

    // 設定後はフィルタリングが有効になる
    let config = ContentFilterConfig {
        pattern: "payload".to_string(),
    };
    filter.set_config(config);
    assert!(filter.filter(&packet));
}

#[test]
fn test_content_filter_invalid_regex() {
    use crate::core::buildings::filters::content_filter::ContentFilterConfig;

    let config = ContentFilterConfig {
        pattern: "[invalid regex".to_string(), // 無効な正規表現
    };
    let filter = ContentFilter::new_with_config(0, Default::default(), 0, config);
    let packet = create_test_packet();

    // 無効な正規表現の場合は何も通さない
    assert!(!filter.filter(&packet));
}

#[test]
fn test_protocol_filter_tcp() {
    use crate::core::buildings::filters::protocol_filter::{ProtocolFilter, ProtocolFilterConfig};

    let config = ProtocolFilterConfig {
        protocol: Protocol::Tcp,
    };
    let filter = ProtocolFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // protocol = Tcp
    let mut packet_no_match = create_test_packet();
    packet_no_match.protocol = Protocol::Udp;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_protocol_filter_udp() {
    use crate::core::buildings::filters::protocol_filter::{ProtocolFilter, ProtocolFilterConfig};

    let config = ProtocolFilterConfig {
        protocol: Protocol::Udp,
    };
    let filter = ProtocolFilter::new_with_config(0, Default::default(), 0, config);

    let mut packet_match = create_test_packet();
    packet_match.protocol = Protocol::Udp;
    let packet_no_match = create_test_packet(); // protocol = Tcp

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_protocol_filter_no_config() {
    use crate::core::buildings::filters::protocol_filter::ProtocolFilter;

    let filter = ProtocolFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 設定がない場合は何も通さない
    assert!(!filter.filter(&packet));
}

#[test]
fn test_protocol_filter_set_config() {
    use crate::core::buildings::filters::protocol_filter::{ProtocolFilter, ProtocolFilterConfig};

    let mut filter = ProtocolFilter::new(0, Default::default(), 0);
    let packet = create_test_packet(); // protocol = Tcp

    // 初期状態では何も通さない
    assert!(!filter.filter(&packet));

    // 設定後はフィルタリングが有効になる
    let config = ProtocolFilterConfig {
        protocol: Protocol::Tcp,
    };
    filter.set_config(config);
    assert!(filter.filter(&packet));
}

#[test]
fn test_port_filter_source_direction() {
    use crate::core::buildings::filters::port_filter::{PortFilterConfig, PortFilterDirection};

    let config = PortFilterConfig {
        target_port: 12345,
        direction: PortFilterDirection::Source,
    };
    let filter = PortFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // source_port = 12345
    let mut packet_no_match = create_test_packet();
    packet_no_match.source_port = 54321;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_port_filter_destination_direction() {
    use crate::core::buildings::filters::port_filter::{PortFilterConfig, PortFilterDirection};

    let config = PortFilterConfig {
        target_port: 80,
        direction: PortFilterDirection::Destination,
    };
    let filter = PortFilter::new_with_config(0, Default::default(), 0, config);

    let packet_match = create_test_packet(); // dest_port = 80
    let mut packet_no_match = create_test_packet();
    packet_no_match.dest_port = 443;

    assert!(filter.filter(&packet_match));
    assert!(!filter.filter(&packet_no_match));
}

#[test]
fn test_port_filter_no_config() {
    let filter = PortFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 設定がない場合は何も通さない
    assert!(!filter.filter(&packet));
}

#[test]
fn test_port_filter_set_config() {
    use crate::core::buildings::filters::port_filter::{PortFilterConfig, PortFilterDirection};

    let mut filter = PortFilter::new(0, Default::default(), 0);
    let packet = create_test_packet();

    // 初期状態では何も通さない
    assert!(!filter.filter(&packet));

    // 設定後はフィルタリングが有効になる
    let config = PortFilterConfig {
        target_port: 12345,
        direction: PortFilterDirection::Source,
    };
    filter.set_config(config);
    assert!(filter.filter(&packet)); // source_port 12345 == config port 12345
}
