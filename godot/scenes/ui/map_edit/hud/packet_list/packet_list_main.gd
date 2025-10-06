extends Control

const PACKET_ITEM_SCENE = preload("res://scenes/ui/map_edit/hud/packet_list/packet_item.tscn")

var all_packets: Array = []
var current_tab: String = "correct"

# パケット一覧を開く（JSONファイルから）
func open_from_json(packets_json_path: String) -> void:
	_load_packets_from_json_path(packets_json_path)
	_switch_tab("correct")
	show()

# パケット一覧を開く（Rustデータから）
func open_from_rust(packets: Array) -> void:
	_load_packets_from_rust(packets)
	_switch_tab("correct")
	show()

# JSONファイルからパケットデータを読み込む
func _load_packets_from_json_path(json_path: String) -> void:
	var file = FileAccess.open(json_path, FileAccess.READ)
	if file == null:
		push_error("Failed to open packets file: " + json_path)
		all_packets = []
		return
	
	var json_text = file.get_as_text()
	file.close()
	
	var json = JSON.new()
	var parse_result = json.parse(json_text)

	if parse_result != OK:
		push_error("Failed to parse packets JSON: " + json_path)
		all_packets = []
		return
	
	var data = json.get_data()
	if data is Dictionary and data.has("packets"):
		all_packets = data["packets"]
		print("Loaded ", all_packets.size(), " packets from ", json_path)
	else:
		push_error("Invalid packets JSON format")
		all_packets = []

# Rustから取得したパケットデータを読み込む
func _load_packets_from_rust(packets: Array) -> void:
	all_packets = []
	
	for packet_dict in packets:
		# 正規化されたパケットデータを作成
		var normalized_packet = {}
		
		# ラベル値を文字列に変換 (1=correct, -1=incorrect, 0=unknown)
		var label_value = packet_dict.get("label", 0)
		var label_str: String
		if label_value == 1:
			label_str = "correct"
		elif label_value == -1:
			label_str = "incorrect"
		else:
			label_str = "unknown"
		
		# フィールド名を正規化してコピー (Rust形式 → JSON形式)
		normalized_packet["src_ip"] = packet_dict.get("source_ip", "N/A")
		normalized_packet["dst_ip"] = packet_dict.get("dest_ip", "N/A")
		normalized_packet["src_port"] = packet_dict.get("source_port", 0)
		normalized_packet["dst_port"] = packet_dict.get("dest_port", 0)
		normalized_packet["protocol"] = packet_dict.get("protocol", 0)
		normalized_packet["size"] = packet_dict.get("length", 0)
		normalized_packet["label"] = label_str
		normalized_packet["payload"] = packet_dict.get("payload", "")
		
		# バックエンドがバグっておりTCPが0 UDPが1 Unknownが2になっている
		print("Original protocol from Rust: ", normalized_packet["protocol"])
		if normalized_packet["protocol"] == 0:
			normalized_packet["protocol"] = 6
		elif normalized_packet["protocol"] == 1:
			normalized_packet["protocol"] = 17
		else:
			normalized_packet["protocol"] = 0

		all_packets.append(normalized_packet)
	
	print("Loaded ", all_packets.size(), " packets from Rust")

# タブを切り替える
func _switch_tab(tab_name: String) -> void:
	current_tab = tab_name
	_update_tab_buttons()
	_populate_packet_list()

# タブボタンの表示を更新
func _update_tab_buttons() -> void:
	var correct_btn = get_node_or_null("%CorrectButton")
	var incorrect_btn = get_node_or_null("%IncorrectButton")
	var unknown_btn = get_node_or_null("%UnknownButton")
	
	if correct_btn:
		correct_btn.disabled = (current_tab == "correct")
	if incorrect_btn:
		incorrect_btn.disabled = (current_tab == "incorrect")
	if unknown_btn:
		unknown_btn.disabled = (current_tab == "unknown")

# パケットリストを更新
func _populate_packet_list() -> void:
	var packet_container = get_node_or_null("%PacketContainer")
	if packet_container == null:
		push_error("PacketContainer not found")
		return
	
	# 既存のアイテムをクリア
	for child in packet_container.get_children():
		child.queue_free()
	
	# 現在のタブに応じてパケットをフィルタリング
	var filtered_packets = []
	for packet in all_packets:
		var label = packet.get("label", "unknown")
		if label == current_tab:
			filtered_packets.append(packet)
	
	print("Displaying ", filtered_packets.size(), " packets for tab: ", current_tab)

	# パケットアイテムを生成して追加
	for packet in filtered_packets:
		var packet_item = PACKET_ITEM_SCENE.instantiate()
		packet_container.add_child(packet_item)
		packet_item.set_packet_data(packet)

# 閉じるボタンが押された時
func _on_close_button_pressed() -> void:
	queue_free()

func _ready() -> void:
	# ボタンのシグナルを接続
	var close_btn = get_node_or_null("%CloseButton")
	if close_btn:
		close_btn.pressed.connect(_on_close_button_pressed)
	
	var correct_btn = get_node_or_null("%CorrectButton")
	if correct_btn:
		correct_btn.pressed.connect(func(): _switch_tab("correct"))
	
	var incorrect_btn = get_node_or_null("%IncorrectButton")
	if incorrect_btn:
		incorrect_btn.pressed.connect(func(): _switch_tab("incorrect"))
	
	var unknown_btn = get_node_or_null("%UnknownButton")
	if unknown_btn:
		unknown_btn.pressed.connect(func(): _switch_tab("unknown"))
	
	# 初期状態では非表示
	hide()
