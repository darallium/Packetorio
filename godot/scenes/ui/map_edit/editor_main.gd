extends Node2D

const CONTENT_SETTING_SCENE = preload("res://scenes/ui/map_edit/hud/building_settings/content_setting.tscn")
const LENGTH_SETTING_SCENE = preload("res://scenes/ui/map_edit/hud/building_settings/length_setting.tscn")
const BUILDING_SETTING_SCENE = preload("res://scenes/ui/map_edit/hud/building_settings/building_settings.tscn")
const PACKET_LIST_SCENE = preload("res://scenes/ui/map_edit/hud/packet_list/packet_list_main.tscn")
const TUTORIAL_POPUP_SCENE = preload("res://scenes/tutorial/TutorialPopup.tscn")
const RESULT_SCENE = preload("res://scenes/ui/map_edit/hud/result_window/result_window.tscn")

@onready var building_layer: TileMapLayer = %Building
@onready var buildplan_grid_overlay_layer: TileMapLayer = %BuildplanGridOverlay
@onready var buildplan_preview_layer: TileMapLayer = %BuildplanPreview
@onready var buildplan_dragging_preview_layer: TileMapLayer = %BuildplanDraggingPreview
@onready var buildplan_dragging_grid_overlay_layer: TileMapLayer = %BuildplanDraggingGridOverlay
@onready var buildplan_mouse_pos_preview_layer: TileMapLayer = %BuildplanMousePosPreview
@onready var list_view = %BuildingList
@onready var header = %header
@onready var deletion_area: ColorRect = %DeletionArea

var placeable_tile_source_id = 0
var placeable_tile_atlas_coords = Vector2i(0, 0)

# 削除マーク用のタイル（BuildplanGridOverlayの削除用）
var removal_mark_source_id = 1  # BuildplanGridOverlayの削除用タイル
var removal_mark_atlas_coords = Vector2i(0, 0)

var is_dragging: bool = false
var is_dragging_removal: bool = false
var drag_start_pos: Vector2i

var map_controller
var _content_setting_ui
var _length_setting_ui
var _building_setting_ui
var _packet_list_popup = null
var _tutorial_popup: Control = null
var _se_player: AudioStreamPlayer
var _place_sound: AudioStream = preload("res://assets/audio/click1.ogg")
var _remove_sound: AudioStream = preload("res://assets/audio/click2.ogg")
var _commit_sound: AudioStream = preload("res://assets/audio/click3.ogg")
var _packet_list_layer: CanvasLayer = null
var _tutorial_popup_layer: CanvasLayer = null

func _ready() -> void:
	# SEプレーヤーの準備
	_se_player = AudioStreamPlayer.new()
	add_child(_se_player)
	_se_player.bus = "SFX"
	
	map_controller = MapController.new()
	add_child(map_controller)
	EditorManager.map_controller = map_controller
	
	map_controller.building_placed.connect(_on_building_placed)
	map_controller.building_removed.connect(_on_building_removed)

	# マップの読み込みと初期化
	_initialize_map()
	
	
	header.update_cost_preview(EditorManager.current_cost)

	_content_setting_ui = CONTENT_SETTING_SCENE.instantiate()
	_add_setting_panel(_content_setting_ui)
	_length_setting_ui = LENGTH_SETTING_SCENE.instantiate()
	_add_setting_panel(_length_setting_ui)
	_building_setting_ui = BUILDING_SETTING_SCENE.instantiate()
	_add_setting_panel(_building_setting_ui)
	
	# BuildplanMousePosPreview の modulate を設定（alpha 50%）
	if buildplan_mouse_pos_preview_layer:
		buildplan_mouse_pos_preview_layer.modulate = Color(1.0, 1.0, 1.0, 0.5)
	
	# ShowPacketsボタンのシグナルを接続
	var show_packets_button = %ShowPackets
	if show_packets_button:
		show_packets_button.pressed.connect(_on_show_packets_pressed)

	# ShowManualボタンのシグナルを接続
	var show_manual_button = %ShowManual
	if show_manual_button:
		show_manual_button.pressed.connect(_show_manual_popup)

	# ShowTutorialボタンのシグナルを接続
	var show_tutorial_button = %ShowTutorial
	if show_tutorial_button:
		show_tutorial_button.pressed.connect(_show_tutorial_popup)

	var quit_button = $%QuitButton
	if quit_button:
		quit_button.pressed.connect(func(): SceneManager.change_scene("res://scenes/ui/stage/Tree.tscn"))
		

	_show_tutorial_popup()

func _process(_delta: float) -> void:
	_update_mouse_pos_preview()
	
	# ゲーム終了判定をチェック
	if EditorManager.check_game_end_condition():
		EditorManager.game_ended = true
		EditorManager.map_controller.stop_the_world()
		EditorManager.stop_game_timer()
		show_result_screen()

func _update_mouse_pos_preview() -> void:
	# 建物が選択されていない場合はクリア
	if not EditorManager.is_building_selected():
		buildplan_mouse_pos_preview_layer.clear()
		return
	
	# マウス位置をタイル座標に変換
	var mouse_pos = get_global_mouse_position()
	var tile_coords = buildplan_mouse_pos_preview_layer.local_to_map(mouse_pos)
	
	# 前回と同じ位置なら更新不要
	# （パフォーマンス最適化）
	
	# レイヤーをクリア
	buildplan_mouse_pos_preview_layer.clear()
	
	# 選択中の建物情報を取得
	var source_id = EditorManager.get_selected_building_id()
	var rotation = EditorManager.get_selected_building_rotation()
	var building_size = get_building_size(source_id)
	
	# コンベアの場合はalternativeを使用
	if source_id == 2:
		buildplan_mouse_pos_preview_layer.set_cell(tile_coords, source_id, Vector2i(0, 0), rotation)
	else:
		# その他のbuildingはatlas coordsを使用
		var atlas_coords = EditorManager.get_atlas_coords_for_rotation(source_id, rotation, building_size)
		buildplan_mouse_pos_preview_layer.set_cell(tile_coords, source_id, atlas_coords, 0)

func _add_setting_panel(panel_instance) -> void:
	var popup_layer = CanvasLayer.new()
	popup_layer.name = "BuildingSettingsPopup"
	panel_instance.anchor_left = 0.5
	panel_instance.anchor_top = 0.5
	panel_instance.anchor_right = 0.5
	panel_instance.anchor_bottom = 0.5
	# setting UIのsizeをハードコードしている -(250 / 2), -(110 / 2)
	panel_instance.size = Vector2(250, 110)
	panel_instance.position = Vector2(-125, -55)
	popup_layer.add_child(panel_instance)
	add_child(popup_layer)
	print("Building settings panel added with fixed size and centered.")

func _create_canvas_layer(layer_name: String, layer_value: int = 1) -> CanvasLayer:
	var layer := CanvasLayer.new()
	layer.name = layer_name
	layer.layer = layer_value
	add_child(layer)
	return layer


func _ensure_tutorial_popup_attached() -> void:
	if not is_instance_valid(_tutorial_popup):
		_tutorial_popup = TUTORIAL_POPUP_SCENE.instantiate()
	if not is_instance_valid(_tutorial_popup_layer):
		_tutorial_popup_layer = _create_canvas_layer("TutorialPopupLayer", 2)
	else:
		_tutorial_popup_layer.layer = 2
	if _tutorial_popup.get_parent() != _tutorial_popup_layer:
		if _tutorial_popup.get_parent():
			_tutorial_popup.get_parent().remove_child(_tutorial_popup)
		_tutorial_popup_layer.add_child(_tutorial_popup)


func _on_building_placed(info: Dictionary):
	var pos = info["pos"]
	var source_id = info["type"]
	var rotation = info["rotation"]
	building_layer.set_cell(pos, source_id, Vector2i(0, 0), rotation)

func _on_building_removed(id: int, pos: Vector2i):
	building_layer.erase_cell(pos)

var _map_metadata

# マップの初期化処理
func _initialize_map() -> void:
	if EditorManager.selected_map_path == "":
		push_warning("No map selected!")
		return
	
	print("Loading map: ", EditorManager.selected_map_path)
	
	# マップをロード
	_map_metadata = map_controller.load_map(EditorManager.selected_map_path)

	if not _map_metadata:
		print("Failed to load map!")
		return
	
	# メタデータからマップサイズを取得
	EditorManager.map_width = _map_metadata.get("width", 0)
	EditorManager.map_height = _map_metadata.get("height", 0)
	
	print("Map loaded successfully!")
	print("  - Width: ", EditorManager.map_width)
	print("  - Height: ", EditorManager.map_height)
	print("  - Map name: ", _map_metadata.get("mapname", "Unknown"))
	
	# 既存の建物を取得して表示
	_load_existing_buildings()
	
	# マップ境界を視覚化
	_draw_map_boundary()

# 既存建物の読み込みと表示
func _load_existing_buildings() -> void:
	var all_buildings = map_controller.get_all_buildings()
	
	print("Loading ", all_buildings.size(), " existing buildings...")
	
	for building_data in all_buildings:
		var pos = building_data.get("pos", Vector2i(0, 0))
		var building_type = building_data.get("type", -1)
		var rotation = building_data.get("rotation", 0)
		
		if building_type != -1:
			building_layer.set_cell(pos, building_type, Vector2i(0, 0), rotation)
			print("  - Loaded building at ", pos, " type: ", building_type, " rotation: ", rotation)
	
	print("Finished loading existing buildings")

# マップ境界線の描画
func _draw_map_boundary() -> void:
	# 既存の境界線があれば削除
	var existing_boundary = get_node_or_null("MapBoundary")
	if existing_boundary:
		existing_boundary.queue_free()
	
	# Line2Dで境界線を描画
	var boundary = Line2D.new()
	boundary.name = "MapBoundary"
	boundary.width = 2.0
	boundary.default_color = Color(1.0, 1.0, 0.0, 0.8)  # 黄色
	boundary.z_index = 100
	
	var tile_size = 16
	var width_px = EditorManager.map_width * tile_size
	var height_px = EditorManager.map_height * tile_size
	
	# 矩形の各頂点を追加
	boundary.add_point(Vector2(0, 0))
	boundary.add_point(Vector2(width_px, 0))
	boundary.add_point(Vector2(width_px, height_px))
	boundary.add_point(Vector2(0, height_px))
	boundary.add_point(Vector2(0, 0))  # 閉じる
	
	add_child(boundary)
	print("Map boundary drawn: ", EditorManager.map_width, "x", EditorManager.map_height, " tiles")

func _normalize_line(start_pos: Vector2i, end_pos: Vector2i) -> Array:
	var points: Array[Vector2i] = []
	var dx = abs(start_pos.x - end_pos.x)
	var dy = abs(start_pos.y - end_pos.y)
	var direction: int = 0
	
	if dx > dy:
		var sign_x = sign(end_pos.x - start_pos.x)
		for i in range(dx + 1):
			points.append(Vector2i(start_pos.x + i * sign_x, start_pos.y))
		direction = 0 if sign_x == 1 else 2
	else:
		var sign_y = sign(end_pos.y - start_pos.y)
		for i in range(dy + 1):
			points.append(Vector2i(start_pos.x, start_pos.y + i * sign_y))
		direction = 1 if sign_y == 1 else 3
	
	return [points, direction]

func _get_rect_cells(start_pos: Vector2i, end_pos: Vector2i) -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	var min_x = min(start_pos.x, end_pos.x)
	var max_x = max(start_pos.x, end_pos.x)
	var min_y = min(start_pos.y, end_pos.y)
	var max_y = max(start_pos.y, end_pos.y)
	
	for x in range(min_x, max_x + 1):
		for y in range(min_y, max_y + 1):
			cells.append(Vector2i(x, y))
	
	return cells

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("commit_plan"):
		_commit_building_plan()
		get_viewport().set_input_as_handled()
		return
	
	if event.is_action_pressed("cancel_plan"):
		_cancel_building_plan()
		get_viewport().set_input_as_handled()
		return
	
	# Rキーで回転
	if event.is_action_pressed("rotate_building"):
		if EditorManager.is_building_selected():
			EditorManager.rotate_selected_building()
			get_viewport().set_input_as_handled()
			return
	
	var tile_coords: Vector2i
	
	if event is InputEventMouseButton:
		tile_coords = buildplan_dragging_preview_layer.local_to_map(get_global_mouse_position())
		
		if event.button_index == MOUSE_BUTTON_LEFT:
			if event.is_pressed():
				_handle_left_click_press(tile_coords)
			else:
				_handle_left_click_release(tile_coords)
		
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			if event.is_pressed():
				_handle_right_click_press(tile_coords)
			else:
				_handle_right_click_release(tile_coords)
		
		# マウスホイールでも回転（上下両方向で回転実行）
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP or event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			if EditorManager.is_building_selected():
				EditorManager.rotate_selected_building()
				get_viewport().set_input_as_handled()
	
	if event is InputEventMouseMotion:
		if is_dragging:
			_handle_mouse_drag(tile_coords)
		elif is_dragging_removal:
			_handle_removal_drag(tile_coords)

func show_result_screen():
	print("Showing result screen...")
	
	# 既にリザルト画面が表示されている場合は何もしない
	if $HUD.has_node("ResultScreen"):
		return
	
	# リザルト画面をインスタンス化
	var result_screen = RESULT_SCENE.instantiate()
	result_screen.name = "ResultScreen"
	
	# HUDに追加して表示
	$HUD.add_child(result_screen)

func _handle_left_click_press(tile_coords: Vector2i) -> void:
	if EditorManager.is_building_selected():
		var source_id = EditorManager.get_selected_building_id()
		
		# 配置可能性チェック
		if not _can_place_building(tile_coords, source_id):
			print("Cannot place building at: ", tile_coords, " (overlaps with existing building)")
			return
		
		drag_start_pos = tile_coords
		is_dragging = true
		var rotation = EditorManager.get_selected_building_rotation()
		var building_size = get_building_size(source_id)
		
		# コンベアはalternativeで、その他はatlas coordsで回転管理
		if source_id == 2:
			buildplan_dragging_preview_layer.set_cell(tile_coords, source_id, Vector2i(0, 0), rotation)
		else:
			var atlas_coords = EditorManager.get_atlas_coords_for_rotation(source_id, rotation, building_size)
			buildplan_dragging_preview_layer.set_cell(tile_coords, source_id, atlas_coords, 0)
		
		# 占有セルにもグリッドオーバーレイを表示
		var occupied = _get_occupied_cells(tile_coords, source_id)
		for occupied_cell in occupied:
			buildplan_dragging_grid_overlay_layer.set_cell(occupied_cell, placeable_tile_source_id, placeable_tile_atlas_coords)
	else:
		var existing_building_source_id = building_layer.get_cell_source_id(tile_coords)
		if _content_setting_ui: _content_setting_ui.hide()
		if _length_setting_ui: _length_setting_ui.hide()
		if _building_setting_ui: _building_setting_ui.hide()
		if existing_building_source_id != -1:
			var building_id = EditorManager.find_building_id_by_tile(tile_coords)
			if building_id >= 0:
				match existing_building_source_id:
					10, 11, 13: # IP, Port, Protocol Filter
						if _building_setting_ui:
							_building_setting_ui.open(building_id, existing_building_source_id)
					12: # Length Filter
						if _length_setting_ui:
							_length_setting_ui.open(building_id)
					14: # Content Filter
						if _content_setting_ui:
							_content_setting_ui.open(building_id)
		#if existing_building_source_id != -1:
			#var building_id = EditorManager.find_building_id_by_tile(tile_coords)
			#if building_id >= 0:
				## Debug print removed for production
				#if existing_building_source_id == 12:
					#if _content_setting_ui:
						#_content_setting_ui.hide()
					#if _length_setting_ui:
						#_length_setting_ui.open(building_id)
				#elif existing_building_source_id == 14:
					#if _length_setting_ui:
						#_length_setting_ui.hide()
					#if _content_setting_ui:
						#_content_setting_ui.open(building_id)
				#else:
					#if _content_setting_ui:
						#_content_setting_ui.hide()
					#if _length_setting_ui:
						#_length_setting_ui.hide()
			#else:
				#if _content_setting_ui:
					#_content_setting_ui.hide()
				#if _length_setting_ui:
					#_length_setting_ui.hide()
		#else:
			#if _content_setting_ui:
				#_content_setting_ui.hide()
			#if _length_setting_ui:
				#_length_setting_ui.hide()

func _handle_left_click_release(tile_coords: Vector2i) -> void:
	if not is_dragging:
		return
	
	is_dragging = false
	
	if tile_coords != Vector2i(-1, -1):
		var cells = buildplan_dragging_preview_layer.get_used_cells()
		for cell in cells:
			var source_id = buildplan_dragging_preview_layer.get_cell_source_id(cell)
			
			# 再度配置可能性チェック（ドラッグ中に状況が変わった可能性がある）
			if not _can_place_building(cell, source_id):
				print("Cannot place building at: ", cell, " (overlaps detected at release)")
				continue
			
			# atlas_coordsとalternative_tileをそのまま転送（回転状態を保持）
			var atlas_coords = buildplan_dragging_preview_layer.get_cell_atlas_coords(cell)
			var alternative_tile = buildplan_dragging_preview_layer.get_cell_alternative_tile(cell)
			buildplan_preview_layer.set_cell(cell, source_id, atlas_coords, alternative_tile)
			
			# 占有セルにもグリッドオーバーレイを表示
			var occupied = _get_occupied_cells(cell, source_id)
			for occupied_cell in occupied:
				buildplan_grid_overlay_layer.set_cell(occupied_cell, placeable_tile_source_id, placeable_tile_atlas_coords)
		
		if not cells.is_empty():
			_play_se(_place_sound)
		buildplan_dragging_grid_overlay_layer.clear()
		buildplan_dragging_preview_layer.clear()
		_recalculate_and_update_preview_cost()

func _handle_right_click_press(tile_coords: Vector2i) -> void:
	if EditorManager.is_building_selected():
		# 選択状態なら選択解除
		EditorManager.deselect_building()
		list_view.visible = false
	else:
		# 削除モード開始（マークは付けずにドラッグ開始位置のみ記録）
		drag_start_pos = tile_coords
		is_dragging_removal = true
		# ColorRectの初期表示（開始位置から）
		_update_deletion_area(drag_start_pos, tile_coords)

func _handle_right_click_release(tile_coords: Vector2i) -> void:
	if is_dragging_removal:
		is_dragging_removal = false
		deletion_area.visible = false
		
		tile_coords = buildplan_dragging_preview_layer.local_to_map(get_global_mouse_position())
		if tile_coords == Vector2i(-1, -1):
			return
		
		# リリース時に範囲内のすべてのセルに削除マークを付ける
		var rect_cells = _get_rect_cells(drag_start_pos, tile_coords)
		if not rect_cells.is_empty():
			_play_se(_remove_sound)
		for point in rect_cells:
			_mark_cell_for_removal(point)
		
		print("Removal area finalized: ", rect_cells.size(), " cells")

func _handle_removal_drag(tile_coords: Vector2i) -> void:
	tile_coords = buildplan_dragging_preview_layer.local_to_map(get_global_mouse_position())
	
	if tile_coords == Vector2i(-1, -1):
		return
	
	# ColorRectのみ更新（削除マークは付けない）
	_update_deletion_area(drag_start_pos, tile_coords)

# DeletionArea ColorRectを更新する関数
func _update_deletion_area(start: Vector2i, end: Vector2i) -> void:
	if not deletion_area:
		return
	
	# タイル座標から実際のピクセル位置を計算
	# map_to_localはタイルの中心を返すので、左上コーナーに調整
	var tile_size = 16
	
	# 各タイルの左上コーナーのワールド座標を計算
	var start_world = buildplan_dragging_preview_layer.map_to_local(start) - Vector2(tile_size/2, tile_size/2)
	var end_world = buildplan_dragging_preview_layer.map_to_local(end) - Vector2(tile_size/2, tile_size/2)
	
	# 矩形の左上と右下を計算
	var min_x = min(start_world.x, end_world.x)
	var min_y = min(start_world.y, end_world.y)
	var max_x = max(start_world.x, end_world.x)
	var max_y = max(start_world.y, end_world.y)
	
	# 位置とサイズを設定（右下のタイルも含める）
	deletion_area.position = Vector2(min_x, min_y)
	deletion_area.size = Vector2(max_x - min_x + tile_size, max_y - min_y + tile_size)
	deletion_area.visible = true

func _mark_cell_for_removal(tile_coords: Vector2i) -> void:
	# まず直接そのセルに建物があるかチェック
	var direct_building_id = building_layer.get_cell_source_id(tile_coords)
	var direct_preview_id = buildplan_preview_layer.get_cell_source_id(tile_coords)
	
	var found_building = false
	
	# 既存建物の処理
	if direct_building_id != -1:
		# アンカーセルを特定
		var anchor_cell = _find_building_anchor(tile_coords)
		var building_id = building_layer.get_cell_source_id(anchor_cell)

		# インターネット(0)とデータセンター(1)は削除不可
		if building_id == 0 or building_id == 1:
			print("Cannot remove this building.")
			return

		var occupied = _get_occupied_cells(anchor_cell, building_id)
		
		# 占有領域全体を削除マーク
		for cell in occupied:
			if not EditorManager.buildings_to_remove.has(cell):
				EditorManager.buildings_to_remove.append(cell)
				buildplan_grid_overlay_layer.set_cell(cell, removal_mark_source_id, removal_mark_atlas_coords)
		
		print("Marked building for removal at anchor: ", anchor_cell, " (occupied cells: ", occupied.size(), ")")
		found_building = true
	else:
		# 直接建物がない場合、周辺のアンカーセルを探索
		# マルチブロックの占有セルである可能性を考慮
		var max_building_size = 3  # 最大建物サイズ（余裕を持たせる）
		
		for dx in range(-max_building_size + 1, 1):
			for dy in range(-max_building_size + 1, 1):
				var potential_anchor = tile_coords + Vector2i(dx, dy)
				var anchor_id = building_layer.get_cell_source_id(potential_anchor)
				
				if anchor_id != -1:
					# インターネット(0)とデータセンター(1)は削除不可
					if anchor_id == 0 or anchor_id == 1:
						print("Cannot remove this building.")
						return

					# このアンカーの占有セルを取得
					var occupied = _get_occupied_cells(potential_anchor, anchor_id)
					
					# クリックされたセルが占有領域に含まれているか
					if occupied.has(tile_coords):
						# 占有領域全体を削除マーク
						for cell in occupied:
							if not EditorManager.buildings_to_remove.has(cell):
								EditorManager.buildings_to_remove.append(cell)
								buildplan_grid_overlay_layer.set_cell(cell, removal_mark_source_id, removal_mark_atlas_coords)
						
						print("Marked building for removal at anchor: ", potential_anchor, " (occupied cells: ", occupied.size(), ", clicked: ", tile_coords, ")")
						found_building = true
						break
			
			if found_building:
				break
	
	# プレビュー建物の処理
	if not found_building and direct_preview_id != -1:
		var occupied = _get_occupied_cells(tile_coords, direct_preview_id)
		
		for cell in occupied:
			buildplan_preview_layer.erase_cell(cell)
			var overlay_source = buildplan_grid_overlay_layer.get_cell_source_id(cell)
			if overlay_source != removal_mark_source_id:
				buildplan_grid_overlay_layer.erase_cell(cell)
		
		print("Removed preview building at: ", tile_coords, " (occupied cells: ", occupied.size(), ")")
		found_building = true
	elif not found_building:
		# プレビューレイヤーでも周辺探索
		var max_building_size = 3
		
		for dx in range(-max_building_size + 1, 1):
			for dy in range(-max_building_size + 1, 1):
				var potential_anchor = tile_coords + Vector2i(dx, dy)
				var anchor_id = buildplan_preview_layer.get_cell_source_id(potential_anchor)
				
				if anchor_id != -1:
					var occupied = _get_occupied_cells(potential_anchor, anchor_id)
					
					if occupied.has(tile_coords):
						for cell in occupied:
							buildplan_preview_layer.erase_cell(cell)
							var overlay_source = buildplan_grid_overlay_layer.get_cell_source_id(cell)
							if overlay_source != removal_mark_source_id:
								buildplan_grid_overlay_layer.erase_cell(cell)
						
						print("Removed preview building at anchor: ", potential_anchor, " (occupied cells: ", occupied.size(), ", clicked: ", tile_coords, ")")
						break
			
			if found_building:
				break
	
	_recalculate_and_update_preview_cost()

func _handle_mouse_drag(tile_coords: Vector2i) -> void:
	tile_coords = buildplan_dragging_preview_layer.local_to_map(get_global_mouse_position())
	
	if tile_coords == Vector2i(-1, -1):
		return
	
	var source_id = EditorManager.get_selected_building_id()
	var base_rotation = EditorManager.get_selected_building_rotation()
	
	if source_id == -1:
		return
	
	buildplan_dragging_grid_overlay_layer.clear()
	buildplan_dragging_preview_layer.clear()
	
	var result = _normalize_line(drag_start_pos, tile_coords)
	var line_points: Array[Vector2i] = result[0]
	var line_direction: int = result[1]
	
	var tile_set = buildplan_dragging_preview_layer.get_tile_set()
	var building_size = get_building_size(source_id)
	
	for point in line_points:
		# マルチブロック建物の配置可能性チェック
		if not _can_place_building(point, source_id):
			continue  # 配置不可ならスキップ
		
		var rotation = base_rotation
		var atlas_coords = Vector2i(0, 0)
		
		# コンベアはライン方向に自動回転、alternativeを使用
		if source_id == 2:
			rotation = line_direction
			
			if tile_set:
				var source = tile_set.get_source(source_id)
				if source and not source.has_alternative_tile(Vector2i(0, 0), rotation):
					rotation = 0
			
			buildplan_dragging_preview_layer.set_cell(point, source_id, Vector2i(0, 0), rotation)
		else:
			# その他のbuildingは選択された回転状態を使用、atlas coordsで管理
			atlas_coords = EditorManager.get_atlas_coords_for_rotation(source_id, base_rotation, building_size)
			buildplan_dragging_preview_layer.set_cell(point, source_id, atlas_coords, 0)
		
		# 占有セルにもグリッドオーバーレイを表示
		var occupied = _get_occupied_cells(point, source_id)
		for occupied_cell in occupied:
			buildplan_dragging_grid_overlay_layer.set_cell(occupied_cell, placeable_tile_source_id, placeable_tile_atlas_coords)
	
	_recalculate_and_update_preview_cost()

func _commit_building_plan():
	_play_se(_commit_sound)
	var total_cost = _get_total_preview_cost()
	
	# コストを加算（上限チェックなし）
	EditorManager.add_cost(total_cost)
	
	# 配置されたコンベアと影響を受けるコンベアのセルを記録
	var affected_conveyor_cells: Array[Vector2i] = []
	
	# 削除マークされた建物の削除を先に実行
	var processed_anchors: Array[Vector2i] = []  # 処理済みアンカー
	
	for cell in EditorManager.buildings_to_remove:
		# アンカーセルを特定
		var anchor = _find_building_anchor(cell)
		
		# すでに処理済みのアンカーならスキップ
		if processed_anchors.has(anchor):
			continue
		processed_anchors.append(anchor)
		
		var building_id = building_layer.get_cell_source_id(anchor)
		var occupied = _get_occupied_cells(anchor, building_id)
		
		# 削除する建物のコストを取得して減算
		var building_data = find_building_data_by_id(building_id)
		if building_data:
			var building_cost = building_data.get("cost", 0)
			EditorManager.subtract_cost(building_cost)
		
		# 削除前に占有領域周囲のコンベアをチェック（8方向）
		for occupied_cell in occupied:
			for dx in range(-1, 2):
				for dy in range(-1, 2):
					if dx == 0 and dy == 0:
						continue
					var neighbor = occupied_cell + Vector2i(dx, dy)
					var neighbor_id = building_layer.get_cell_source_id(neighbor)
					if neighbor_id == 2 and not affected_conveyor_cells.has(neighbor):
						affected_conveyor_cells.append(neighbor)
			
			# Building Layerから削除
			building_layer.erase_cell(occupied_cell)
		
		# MapController APIに削除通知（アンカーのみ）
		map_controller.remove_building(anchor)
		
		print("Building removed at anchor: ", anchor, " (occupied cells: ", occupied.size(), ")")
	
	# 新規建物の配置
	for cell in buildplan_preview_layer.get_used_cells():
		var source_id = buildplan_preview_layer.get_cell_source_id(cell)
		var atlas_coords = buildplan_preview_layer.get_cell_atlas_coords(cell)
		var alternative_tile = buildplan_preview_layer.get_cell_alternative_tile(cell)
		
		# Building Layerに直接設定（回転状態を保持）
		# コンベア: alternative_tileで回転管理
		# その他: atlas_coordsで回転管理
		building_layer.set_cell(cell, source_id, atlas_coords, alternative_tile)
		
		# 回転状態を計算
		# コンベアの場合はalternative_tileをそのまま使用
		# その他の場合はatlas_coordsから回転を計算
		var rotation_state = alternative_tile
		if source_id != 2:  # コンベア以外の場合
			var building_size = get_building_size(source_id)
			rotation_state = atlas_coords.x / building_size.x  # x座標のオフセットから回転を計算
		
		# MapController APIに通知（バックエンド側の状態管理用）
		map_controller.place_building(cell, source_id, rotation_state)
		
		# コンベアの場合は記録
		if source_id == 2:
			affected_conveyor_cells.append(cell)
		
		# 新規building周辺のコンベアも更新対象に追加（マルチブロック対応）
		var occupied = _get_occupied_cells(cell, source_id)
		for occupied_cell in occupied:
			# 8方向をチェック（上下左右＋斜め）
			for dx in range(-1, 2):
				for dy in range(-1, 2):
					if dx == 0 and dy == 0:
						continue
					var neighbor = occupied_cell + Vector2i(dx, dy)
					var neighbor_id = building_layer.get_cell_source_id(neighbor)
					if neighbor_id == 2 and not affected_conveyor_cells.has(neighbor):
						affected_conveyor_cells.append(neighbor)
		
		print("Building placed at: ", cell, " type: ", source_id, " atlas: ", atlas_coords, " rotation: ", rotation_state)
	
	# コンベアの自動接続を更新
	_update_conveyor_connections(affected_conveyor_cells)
	
	# クリーンアップ
	buildplan_preview_layer.clear()
	buildplan_grid_overlay_layer.clear()
	EditorManager.buildings_to_remove.clear()
	
	header.update_cost_preview(EditorManager.current_cost)
	print("Building plan committed!")

func _cancel_building_plan():
	buildplan_preview_layer.clear()
	buildplan_grid_overlay_layer.clear()
	buildplan_dragging_preview_layer.clear()
	buildplan_dragging_grid_overlay_layer.clear()
	
	# 削除関連のクリア
	EditorManager.buildings_to_remove.clear()
	deletion_area.visible = false
	
	header.update_cost_preview(EditorManager.current_cost)
	print("Building plan cancelled!")

func _get_total_preview_cost() -> int:
	var total_cost = 0
	for cell in buildplan_preview_layer.get_used_cells():
		var source_id = buildplan_preview_layer.get_cell_source_id(cell)
		var building_data = find_building_data_by_id(source_id)
		if building_data:
			total_cost += building_data.get("cost", 0)
	return total_cost

func find_building_data_by_id(id: int) -> Dictionary:
	for genre_key in list_view.BUILDING_DATA:
		var buildings_in_genre = list_view.BUILDING_DATA[genre_key]
		for building in buildings_in_genre:
			if building.get("id") == id:
				return building
	return {}

# 建物サイズを取得
func get_building_size(building_id: int) -> Vector2i:
	var building_data = find_building_data_by_id(building_id)
	return building_data.get("size", Vector2i(1, 1))

# 建物が占有するすべてのセルを取得（アンカー位置から）
func _get_occupied_cells(anchor_pos: Vector2i, building_id: int) -> Array[Vector2i]:
	var size = get_building_size(building_id)
	var cells: Array[Vector2i] = []
	for x in range(size.x):
		for y in range(size.y):
			cells.append(anchor_pos + Vector2i(x, y))
	return cells

# 指定セルが既存建物の占有範囲に含まれているかチェック
func _is_cell_occupied_by_existing_building(cell: Vector2i) -> bool:
	var existing_anchors = building_layer.get_used_cells()
	for anchor in existing_anchors:
		var anchor_id = building_layer.get_cell_source_id(anchor)
		if anchor_id == -1:
			continue
		var occupied = _get_occupied_cells(anchor, anchor_id)
		if occupied.has(cell):
			return true
	return false

# マルチブロック建物配置時の配置可能性チェック
func _can_place_building(anchor_pos: Vector2i, building_id: int) -> bool:
	var occupied = _get_occupied_cells(anchor_pos, building_id)
	
	for cell in occupied:
		
		# マップ範囲チェック
		if EditorManager.map_width > 0 and EditorManager.map_height > 0:
			if cell.x < 0 or cell.x >= EditorManager.map_width:
				return false
			if cell.y < 0 or cell.y >= EditorManager.map_height:
				return false

		# 既存建物との重複チェック（マルチブロック対応）- 配置不可
		if _is_cell_occupied_by_existing_building(cell):
			return false
		
		# コミット済みプレビュー建物との重複は上書き（削除してから配置）
		var preview_cells = buildplan_preview_layer.get_used_cells()
		for preview_anchor in preview_cells:
			var preview_id = buildplan_preview_layer.get_cell_source_id(preview_anchor)
			var preview_occupied = _get_occupied_cells(preview_anchor, preview_id)
			if preview_occupied.has(cell):
				# プレビュー建物を削除
				for preview_cell in preview_occupied:
					buildplan_preview_layer.erase_cell(preview_cell)
					var overlay_source = buildplan_grid_overlay_layer.get_cell_source_id(preview_cell)
					if overlay_source != removal_mark_source_id:
						buildplan_grid_overlay_layer.erase_cell(preview_cell)
		
		# ドラッグ中のプレビュー建物との重複チェック
		var dragging_cells = buildplan_dragging_preview_layer.get_used_cells()
		for dragging_anchor in dragging_cells:
			# 自分自身は除外
			if dragging_anchor == anchor_pos:
				continue
			
			var dragging_id = buildplan_dragging_preview_layer.get_cell_source_id(dragging_anchor)
			var dragging_occupied = _get_occupied_cells(dragging_anchor, dragging_id)
			if dragging_occupied.has(cell):
				return false
	
	return true

# 指定セルがどの建物のアンカーか検索（マルチブロック対応）
func _find_building_anchor(cell: Vector2i) -> Vector2i:
	# まず自分がアンカーかチェック
	var source_id = building_layer.get_cell_source_id(cell)
	if source_id != -1:
		var size = get_building_size(source_id)
		if size == Vector2i(1, 1):
			return cell  # 1x1ならアンカーは自分
		
		# マルチブロックの場合、周囲を探索
		for dx in range(-size.x + 1, 1):
			for dy in range(-size.y + 1, 1):
				var potential_anchor = cell + Vector2i(dx, dy)
				var anchor_source = building_layer.get_cell_source_id(potential_anchor)
				if anchor_source == source_id:
					# このアンカー候補から占有セルを計算
					var occupied = _get_occupied_cells(potential_anchor, anchor_source)
					if occupied.has(cell):
						return potential_anchor
	
	return cell  # 見つからない場合は自分を返す

func _update_conveyor_connections(changed_cells: Array[Vector2i]) -> void:
	# 変更されたセルとその周囲のコンベアを更新
	var cells_to_update: Array[Vector2i] = []
	
	for cell in changed_cells:
		if not cells_to_update.has(cell):
			cells_to_update.append(cell)
		
		# 周囲8方向もチェック
		for dx in range(-1, 2):
			for dy in range(-1, 2):
				if dx == 0 and dy == 0:
					continue
				var neighbor = cell + Vector2i(dx, dy)
				var neighbor_id = building_layer.get_cell_source_id(neighbor)
				if neighbor_id == 2 and not cells_to_update.has(neighbor):
					cells_to_update.append(neighbor)
	
	# 各コンベアの接続状況を更新
	for cell in cells_to_update:
		var source_id = building_layer.get_cell_source_id(cell)
		if source_id != 2:  # コンベアでない場合はスキップ
			continue
		
		var rotation = building_layer.get_cell_alternative_tile(cell)
		var connections = _check_conveyor_connections(cell, rotation)
		var result = _get_atlas_and_alt_for_connections(connections, rotation)
		var new_atlas = result[0]
		var new_alt = result[1]
		
		# Atlas座標とalternative_tileを更新
		building_layer.set_cell(cell, 2, new_atlas, new_alt)
		print("Updated conveyor at ", cell, " atlas: ", new_atlas, " alt: ", new_alt)

func _check_conveyor_connections(pos: Vector2i, rotation: int) -> Dictionary:
	# rotation（alternative_tile）から基本回転を取得（0-3）
	var base_rotation = rotation % 4
	
	# 絶対座標での4方向の接続をチェック
	var abs_connections = {
		Vector2i.LEFT: false,
		Vector2i.RIGHT: false,
		Vector2i.UP: false,
		Vector2i.DOWN: false
	}
	
	# 出力方向を取得
	var output_dir = _get_output_direction(base_rotation)
	
	# 4方向をチェック（出力方向以外）
	for dir in [Vector2i.LEFT, Vector2i.RIGHT, Vector2i.UP, Vector2i.DOWN]:
		if dir == output_dir:
			continue  # 出力方向は入力として使わない
		
		var neighbor_pos = pos + dir
		var neighbor_id = building_layer.get_cell_source_id(neighbor_pos)
		
		if neighbor_id == -1:
			continue
		
		var is_connected = false
		
		if neighbor_id == 2:  # コンベアの場合
			var neighbor_alt = building_layer.get_cell_alternative_tile(neighbor_pos)
			var neighbor_base_rotation = neighbor_alt % 4
			var neighbor_output_dir = _get_output_direction(neighbor_base_rotation)
			# 隣のコンベアの出力がこちらを向いているか
			is_connected = (neighbor_output_dir == -dir)
		else:
			# 他の建物との接続をチェック（マルチブロック対応）
			# セルに建物があるか、またはマルチブロック建物の占有セルかを確認
			is_connected = _is_cell_occupied_by_existing_building(neighbor_pos)
		
		if is_connected:
			abs_connections[dir] = true
	
	# 絶対座標の接続を相対座標に変換
	var rel_connections = _abs_to_rel_connections(abs_connections, base_rotation)
	
	return rel_connections

func _abs_to_rel_connections(abs_conn: Dictionary, base_rotation: int) -> Dictionary:
	# 絶対方向から相対方向への変換
	# left = 入力方向（出力の逆）, up = 左側, down = 右側
	
	var rel_conn = {
		"left": false,
		"up": false,
		"down": false
	}
	
	match base_rotation:
		0:  # 右向き
			rel_conn["left"] = abs_conn[Vector2i.LEFT]   # 左から = 後ろから
			rel_conn["up"] = abs_conn[Vector2i.UP]       # 上から = 左側から
			rel_conn["down"] = abs_conn[Vector2i.DOWN]   # 下から = 右側から
		1:  # 下向き
			rel_conn["left"] = abs_conn[Vector2i.UP]     # 上から = 後ろから
			rel_conn["up"] = abs_conn[Vector2i.LEFT]     # 左から = 左側から
			rel_conn["down"] = abs_conn[Vector2i.RIGHT]  # 右から = 右側から
		2:  # 左向き
			rel_conn["left"] = abs_conn[Vector2i.RIGHT]  # 右から = 後ろから
			rel_conn["up"] = abs_conn[Vector2i.DOWN]     # 下から = 左側から
			rel_conn["down"] = abs_conn[Vector2i.UP]     # 上から = 右側から
		3:  # 上向き
			rel_conn["left"] = abs_conn[Vector2i.DOWN]   # 下から = 後ろから
			rel_conn["up"] = abs_conn[Vector2i.RIGHT]    # 右から = 左側から
			rel_conn["down"] = abs_conn[Vector2i.LEFT]   # 左から = 右側から
	
	return rel_conn

func _get_output_direction(rotation: int) -> Vector2i:
	match rotation:
		0: return Vector2i.RIGHT
		1: return Vector2i.DOWN
		2: return Vector2i.LEFT
		3: return Vector2i.UP
		_: return Vector2i.ZERO

func _get_atlas_and_alt_for_connections(connections: Dictionary, current_rotation: int) -> Array:
	# 相対接続パターン（コンベアの向き基準）
	# "left" = 後方から, "up" = 左側から, "down" = 右側から
	var has_left = connections["left"]
	var has_up = connections["up"]
	var has_down = connections["down"]
	
	var atlas = Vector2i(0, 0)
	var base_rotation = current_rotation % 4
	var alt = base_rotation
	
	# 接続数でAtlasを選択
	var connection_count = int(has_left) + int(has_up) + int(has_down)
	
	if connection_count == 3:
		# 3方向接続: Y=2（T字）
		atlas = Vector2i(0, 2)
		alt = base_rotation
	elif connection_count == 2:
		if has_left and has_up and not has_down:
			# 後方+左側: Y=1, Alt=4-7
			atlas = Vector2i(0, 1)
			alt = 4 + base_rotation
		elif has_left and has_down and not has_up:
			# 後方+右側: Y=1, Alt=0-3
			atlas = Vector2i(0, 1)
			alt = base_rotation
		elif has_up and has_down and not has_left:
			# 左側+右側（後方なし）: Y=3（直交）
			atlas = Vector2i(0, 3)
			alt = base_rotation
		else:
			# フォールバック
			atlas = Vector2i(0, 0)
			alt = base_rotation
	elif connection_count == 1:
		if has_left:
			# 後方のみ: Y=0（ストレート）
			atlas = Vector2i(0, 0)
			alt = base_rotation
		elif has_up:
			# 左側のみ（L字）: Y=1, Alt=4-7
			atlas = Vector2i(0, 1)
			alt = 4 + base_rotation
		elif has_down:
			# 右側のみ（L字）: Y=1, Alt=0-3
			atlas = Vector2i(0, 1)
			alt = base_rotation
		else:
			# フォールバック
			atlas = Vector2i(0, 0)
			alt = base_rotation
	else:
		# 接続なし
		atlas = Vector2i(0, 0)
		alt = base_rotation
	
	return [atlas, alt]

func _recalculate_and_update_preview_cost():
	var cost_of_committed_preview = 0
	for cell in buildplan_preview_layer.get_used_cells():
		var source_id = buildplan_preview_layer.get_cell_source_id(cell)
		var building_data = find_building_data_by_id(source_id)
		if building_data:
			cost_of_committed_preview += building_data.get("cost", 0)
	
	var cost_of_dragging_preview = 0
	for cell in buildplan_dragging_preview_layer.get_used_cells():
		var source_id = buildplan_dragging_preview_layer.get_cell_source_id(cell)
		var building_data = find_building_data_by_id(source_id)
		if building_data:
			cost_of_dragging_preview += building_data.get("cost", 0)
	
	var total_previewed_cost = cost_of_committed_preview + cost_of_dragging_preview
	# 加算方式：現在のコスト + プレビュー中のコスト
	var final_preview_cost = EditorManager.current_cost + total_previewed_cost
	
	header.update_cost_preview(final_preview_cost)

# ShowPacketsボタンが押された時
func _on_show_packets_pressed() -> void:
	if _packet_list_popup:
		return  # 既に開いている

	_packet_list_popup = PACKET_LIST_SCENE.instantiate()
	_packet_list_layer = _create_canvas_layer("PacketListLayer")
	_packet_list_layer.add_child(_packet_list_popup)

	# パケットJSONパスを渡す
	if EditorManager.selected_packets_path != "":
		_packet_list_popup.open_from_json(EditorManager.selected_packets_path)
	else:
		push_warning("No packets path selected")
	
	# 閉じられたときの処理
	_packet_list_popup.tree_exiting.connect(_on_packet_list_closed)

# パケットリストポップアップが閉じられた時
func _on_packet_list_closed() -> void:
	_packet_list_popup = null
	if is_instance_valid(_packet_list_layer):
		_packet_list_layer.queue_free()
	_packet_list_layer = null

func _play_se(sound: AudioStream) -> void:
	if not sound:
		return
	
	_se_player.stream = sound
	_se_player.pitch_scale = randf_range(0.9, 1.1) # ピッチは少しだけランダム化
	_se_player.play()

func _show_tutorial_popup() -> void:
	if not _map_metadata or not _map_metadata.has("tutorial_title") or not _map_metadata.has("tutorial_message"):
		return  # チュートリアル情報がない

	_ensure_tutorial_popup_attached()

	if is_instance_valid(_tutorial_popup) and _map_metadata:
		var title = _map_metadata.get("tutorial_title", "No title")
		var message = _map_metadata.get("tutorial_message", "No message")
		_tutorial_popup.show_popup(title, message)

const MANUAL_TEXT = \
"""左クリック：建築物の仮配置・操作
右クリック：選択解除
右クリック(ドラッグ)：仮配置キャンセル・削除マーカーの付与

E：仮配置した建築物の確定・削除マーカーが付いた建築物の削除
R：建築物の右回転
マウスホイール：建築物の180度回転
WASD：カメラ移動
"""

func _show_manual_popup() -> void:
	_ensure_tutorial_popup_attached()

	_tutorial_popup.show_popup("操作説明", MANUAL_TEXT)
