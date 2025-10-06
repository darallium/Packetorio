extends Button

# Building情報
var building_id: int = -1
var building_type: int = -1
var building_position: Vector2i = Vector2i.ZERO
var packets_data: Array = []

# 親のresult_windowへの参照
var result_window = null

# セットアップ
func setup(b_id: int, b_type: int, pos: Vector2i, packets: Array, icon: Texture2D, parent_window) -> void:
	building_id = b_id
	building_type = b_type
	building_position = pos
	packets_data = packets
	result_window = parent_window
	
	# アイコン設定（ノードが存在する場合）
	var icon_texture = get_node_or_null("HBoxContainer/IconRect")
	if icon_texture and icon:
		icon_texture.texture = icon
	
	# Building名設定
	var type_name = "Datacenter" if b_type == 1 else "Recycle Bin"
	var name_label = get_node_or_null("HBoxContainer/InfoContainer/NameLabel")
	if name_label:
		name_label.text = type_name
	
	# 座標表示
	var pos_label = get_node_or_null("HBoxContainer/InfoContainer/PosLabel")
	if pos_label:
		pos_label.text = "位置: (%d, %d)" % [pos.x, pos.y]

# ボタンがクリックされた時
func _on_pressed() -> void:
	if result_window and result_window.has_method("show_packets_for_building"):
		result_window.show_packets_for_building(building_id, packets_data)

func _ready() -> void:
	pressed.connect(_on_pressed)
