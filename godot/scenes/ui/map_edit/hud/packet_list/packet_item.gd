extends Control

const ProtocolDict = preload("res://scenes/ui/map_edit/hud/packet_list/protocol_dict.gd")

# パケットデータを設定して表示を更新
func set_packet_data(packet: Dictionary) -> void:
	# IPアドレス情報
	var src_ip = packet.get("src_ip", "N/A")
	var dst_ip = packet.get("dst_ip", "N/A")
	var src_port = packet.get("src_port", 0)
	var dst_port = packet.get("dst_port", 0)
	
	# プロトコル情報
	var protocol_num = packet.get("protocol", 0)
	var protocol_name = ProtocolDict.get_protocol_name(protocol_num)
	
	# サイズ情報
	var size = packet.get("size", 0)
	
	# ラベル情報（correct/incorrect/unknown）
	var label = packet.get("label", "unknown")
	
	# payload情報
	var payload = packet.get("payload", "")
	
	# 各ラベルに情報を設定（固有名でアクセス）
	var ip_label = get_node_or_null("%IPLabel")
	if ip_label:
		ip_label.text = "%s:%d → %s:%d" % [src_ip, src_port, dst_ip, dst_port]
	
	var protocol_label = get_node_or_null("%ProtocolLabel")
	if protocol_label:
		protocol_label.text = "Protocol: %s (%d) | Size: %d bytes" % [protocol_name, protocol_num, size]
	
	var payload_label = get_node_or_null("%PayloadLabel")
	if payload_label:
		if payload != "":
			payload_label.text = payload
		else:
			payload_label.text = "(No payload)"
	
	# ラベルに応じて背景色を変更（オプショナル）
	var panel = get_node_or_null("%Panel")
	if panel:
		match label:
			"correct":
				panel.modulate = Color(0.65, 0.70, 0.70, 1.0)  # 緑っぽい
			"incorrect":
				panel.modulate = Color(0.70, 0.65, 0.65, 1.0)  # 赤っぽい
			"unknown":
				panel.modulate = Color(0.6, 0.6, 0.6, 1.0)  # グレー
			_:
				panel.modulate = Color(1.0, 1.0, 1.0, 1.0)  # 白
