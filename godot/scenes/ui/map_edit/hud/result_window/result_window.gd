extends Control

const PACKET_LIST_SCENE = preload("res://scenes/ui/map_edit/hud/packet_list/packet_list_main.tscn")
const DATACENTER_TYPE = 1
const RECYCLE_BIN_TYPE = 9

@onready var score_label: Label = $AnimationPlayer/BottomRightRect2/ScoreLabel
@onready var retry_button: Button = $AnimationPlayer/BottomRightRect2/RetryButton
@onready var stage_button: Button = $AnimationPlayer/BottomRightRect2/StageButton
@onready var buildings_container = $AnimationPlayer/CenterContainer/BuildingItemsContainer
@onready var hud_layer = $HUD

var _packet_list_instance = null

func _ready():
	# スコア表示
	if score_label:
		score_label.text = "Score: %d" % EditorManager.current_score
	
	# ボタン接続
	if retry_button:
		retry_button.pressed.connect(_on_retry_pressed)
	if stage_button:
		stage_button.pressed.connect(_on_stage_select_pressed)
	
	# Building一覧を読み込み
	_load_buildings()

func _load_buildings():
	if not EditorManager.map_controller:
		print("MapController not found")
		return
	
	var result = EditorManager.map_controller.completed_packets()
	print("Completed packets: ", result)
	if result == null:
		print("No completed packets data")
		return
	
	# building_idごとにグループ化
	var buildings_data = {}
	for packet_info in result:
		var b_id = packet_info.get("building_id", -1)
		var b_type = packet_info.get("building_type", -1)
		
		if b_id == -1 or (b_type != DATACENTER_TYPE and b_type != RECYCLE_BIN_TYPE):
			continue
		
		if not buildings_data.has(b_id):
			buildings_data[b_id] = {
				"building_type": b_type,
				"building_id": b_id,
				"packets": [],
				"position": Vector2i.ZERO
			}
		
		buildings_data[b_id]["packets"].append(packet_info)
	
	# 座標情報を取得（get_all_buildingsから）
	var all_buildings = EditorManager.map_controller.get_all_buildings()
	for building_variant in all_buildings:
		var building_dict = building_variant
		var b_id = building_dict.get("id", -1)
		if buildings_data.has(b_id):
			buildings_data[b_id]["position"] = building_dict.get("pos", Vector2i.ZERO)
	
	# building_items_containerに渡して表示
	if buildings_container:
		buildings_container.setup_buildings(buildings_data, self)

# 特定のBuildingのパケットを表示
func show_packets_for_building(building_id: int, packets: Array):
	# 既存のパケット一覧があれば閉じる
	if _packet_list_instance:
		_packet_list_instance.queue_free()
		_packet_list_instance = null
	
	# パケット一覧を表示
	_packet_list_instance = PACKET_LIST_SCENE.instantiate()
	hud_layer.add_child(_packet_list_instance)
	
	# z_indexを高く設定して最前面に表示
	_packet_list_instance.z_index = 100
	
	_packet_list_instance.open_from_rust(packets)

# リトライボタンが押された時
func _on_retry_pressed():
	# EditorManagerをリセット
	EditorManager.reset_editor_state()
	
	# 現在のステージを再読み込み
	var current_map_path = EditorManager.selected_map_path
	if current_map_path != "":
		SceneManager.change_scene("res://scenes/ui/map_edit/editor_main.tscn")
	else:
		print("No map path selected for retry")

# ステージ選択ボタンが押された時
func _on_stage_select_pressed():
	# EditorManagerをリセット
	EditorManager.reset_editor_state()
	
	# ステージ選択画面に遷移
	SceneManager.change_scene("res://scenes/ui/stage/Tree.tscn")
