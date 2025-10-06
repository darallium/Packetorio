extends Node

# プロトコル番号から名前へのマッピング
const PROTOCOL_NAMES = {
	1: "ICMP",
	4: "IPv4",
	6: "TCP",
	12: "PUP",
	17: "UDP",
	41: "IPv6"
}

# プロトコル番号から名前を取得する関数
static func get_protocol_name(protocol_number: int) -> String:
	return PROTOCOL_NAMES.get(protocol_number, "Unknown(%d)" % protocol_number)
