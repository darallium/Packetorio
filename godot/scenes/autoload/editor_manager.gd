class_name EditorManagerClass
extends Node

# 建物選択状態
var selected_building_id: int = -1
var selected_building_rotation: int = 0

# 削除予定建物の管理
var buildings_to_remove: Array[Vector2i] = []

# コスト管理（加算方式：配置された建物のコストの合計）
var current_cost: int = 0

# スコア管理（header.gdから移動）
var current_score: int = 0

# マップ情報
var selected_map_path: String = ""  # 選択されたマップのJSONパス
var selected_packets_path: String = ""  # 選択されたパケットのJSONパス
var map_width: int = 0              # マップの幅
var map_height: int = 0             # マップの高さ

# タイマー関連
var game_timer: float = 120.0       # 残り時間（秒）
var is_timer_running: bool = false  # タイマーが動作中か
var total_packets_count: int = 0    # ステージ全体のパケット総数
var game_ended: bool = false        # ゲーム終了フラグ

# MapControllerインスタンス参照
var map_controller = null

func _ready():
	print("EditorManager initialized!")

func rust(method: StringName, args: Array = []):
	if map_controller:
		return map_controller.callv(method, args)
	return null

func get_filter_rule(building_id: int) -> Dictionary:
	var result = rust("get_filter_rule", [building_id])
	if result == null:
		return {}
	return result

func set_filter_rules(building_id: int, rule: Dictionary) -> void:
	rust("set_filter_rules", [building_id, rule])

func find_building_id_by_tile(tile: Vector2i) -> int:
	var buildings = rust("get_all_buildings")
	if buildings == null:
		return -1
	for entry_variant in buildings:
		var entry = entry_variant
		if entry.get("pos", Vector2i()) == tile:
			return int(entry.get("id", -1))
	return -1

# 建物選択
func select_building(building_id: int) -> void:
	selected_building_id = building_id
	selected_building_rotation = 0
	print("Building selected: ", building_id)

# 選択解除
func deselect_building() -> void:
	selected_building_id = -1
	selected_building_rotation = 0
	print("Building deselected")

# 建物が選択されているか
func is_building_selected() -> bool:
	return selected_building_id >= 0

# 選択中の建物IDを取得
func get_selected_building_id() -> int:
	return selected_building_id

# 選択中の建物の回転を取得
func get_selected_building_rotation() -> int:
	return selected_building_rotation

# 建物回転（全building対応）
func rotate_selected_building() -> void:
	if selected_building_id >= 0:
		selected_building_rotation = (selected_building_rotation + 1) % 4
		print("Building ", selected_building_id, " rotated to: ", selected_building_rotation)

# 回転に応じた atlas coords を計算
# コンベアはalternativeを使うのでVector2i(0,0)を返す
# その他のbuildingはatlas coordsのx座標を変更
func get_atlas_coords_for_rotation(building_id: int, rotation: int, building_size: Vector2i) -> Vector2i:
	# コンベアは常に(0,0) - alternativeで回転管理
	if building_id == 2:
		return Vector2i(0, 0)
	
	# その他のbuildingはatlas coordsで回転管理
	# 1x1: 0,0 -> 1,0 -> 2,0 -> 3,0
	# 2x2: 0,0 -> 2,0 -> 4,0 -> 6,0
	var x_offset = rotation * building_size.x
	return Vector2i(x_offset, 0)

# 選択中の建物の回転に応じた atlas coords を取得
func get_selected_building_atlas_coords(building_size: Vector2i) -> Vector2i:
	return get_atlas_coords_for_rotation(selected_building_id, selected_building_rotation, building_size)

# コスト追加（建物配置時）
func add_cost(amount: int) -> void:
	current_cost += amount
	print("Added ", amount, " cost. Total: ", current_cost)

# コスト減算（建物削除時）
func subtract_cost(amount: int) -> void:
	current_cost -= amount
	if current_cost < 0:
		current_cost = 0
	print("Subtracted ", amount, " cost. Total: ", current_cost)

# コストリセット
func reset_cost():
	current_cost = 0
	print("Cost reset to: ", current_cost)

# タイマーを開始
func start_game_timer() -> void:
	is_timer_running = true
	print("Game timer started: ", game_timer, " seconds")

# タイマーを停止
func stop_game_timer() -> void:
	is_timer_running = false
	print("Game timer stopped")

# タイマーを毎フレーム更新
func update_timer(delta: float) -> void:
	if is_timer_running:
		game_timer -= delta
		if game_timer <= 0.0:
			game_timer = 0.0
			is_timer_running = false
			print("Game timer finished!")

# 残り時間を取得
func get_remaining_time() -> float:
	return game_timer

# パケット総数を読み込み
func load_total_packets_count() -> void:
	if selected_packets_path == "":
		total_packets_count = 0
		return
	
	var file = FileAccess.open(selected_packets_path, FileAccess.READ)
	if file == null:
		push_error("Failed to open packets file: " + selected_packets_path)
		total_packets_count = 0
		return
	
	var json_text = file.get_as_text()
	file.close()
	
	var json = JSON.new()
	var parse_result = json.parse(json_text)
	
	if parse_result != OK:
		push_error("Failed to parse packets JSON: " + selected_packets_path)
		total_packets_count = 0
		return
	
	var data = json.get_data()
	if data is Dictionary and data.has("packets"):
		total_packets_count = data["packets"].size()
		print("Loaded total packets count: ", total_packets_count)
	else:
		push_error("Invalid packets JSON format")
		total_packets_count = 0

# ゲーム終了判定をチェック
func check_game_end_condition() -> bool:
	if game_ended:
		return false  # 既に終了済み
	
	# タイマーが0になった場合
	if game_timer <= 0.0:
		return true
	
	# パケット完了判定
	if map_controller:
		var result = map_controller.completed_packets()
		if result != null:
			var completed_count = result.size()
			if completed_count >= total_packets_count:
				return true
	
	return false

# エディタ状態をリセット（マップ遷移時）
func reset_editor_state() -> void:
	selected_building_id = -1
	selected_building_rotation = 0
	buildings_to_remove.clear()
	current_cost = 0
	current_score = 0
	# タイマー関連リセット
	game_timer = 120.0
	is_timer_running = false
	total_packets_count = 0
	game_ended = false
	# パケット総数を読み込み
	load_total_packets_count()
	print("Editor state reset")
