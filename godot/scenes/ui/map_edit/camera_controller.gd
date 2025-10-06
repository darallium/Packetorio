extends Camera2D

# カメラ移動の定数
const CAMERA_SPEED = 250.0
const TILE_SIZE = 16

func _ready() -> void:
	# シーンが完全に読み込まれた後にカメラを中央に配置
	call_deferred("_center_camera_on_map")

func _process(delta: float) -> void:
	_update_camera_movement(delta)

# マップの中央にカメラを配置
func _center_camera_on_map() -> void:
	if EditorManager.map_width > 0 and EditorManager.map_height > 0:
		var map_center_x = (EditorManager.map_width * TILE_SIZE) / 2.0
		var map_center_y = (EditorManager.map_height * TILE_SIZE) / 2.0
		position = Vector2(map_center_x, map_center_y)
		print("Camera centered at: ", position, " (Map size: ", EditorManager.map_width, "x", EditorManager.map_height, ")")

# WASD入力によるカメラ移動処理
func _update_camera_movement(delta: float) -> void:
	var move_vector = Vector2.ZERO
	
	# WASD入力をチェック
	if Input.is_action_pressed("camera_move_up"):
		move_vector.y -= 1.0
	if Input.is_action_pressed("camera_move_down"):
		move_vector.y += 1.0
	if Input.is_action_pressed("camera_move_left"):
		move_vector.x -= 1.0
	if Input.is_action_pressed("camera_move_right"):
		move_vector.x += 1.0
	
	# 移動ベクトルが存在する場合のみ処理
	if move_vector.length() > 0:
		# 正規化して斜め移動でも速度一定にする
		move_vector = move_vector.normalized()
		
		# カメラ位置を更新
		position += move_vector * CAMERA_SPEED * delta
		
		# マップ範囲内に制限
		_clamp_camera_to_map_bounds()

# カメラ位置をマップ範囲内に制限
func _clamp_camera_to_map_bounds() -> void:
	if EditorManager.map_width > 0 and EditorManager.map_height > 0:
		var max_x = EditorManager.map_width * TILE_SIZE
		var max_y = EditorManager.map_height * TILE_SIZE
		
		# カメラの中心がマップ範囲内に収まるように制限
		position.x = clamp(position.x, 0, max_x)
		position.y = clamp(position.y, 0, max_y)

# 外部からカメラを中央に戻すメソッド（必要に応じて使用）
func center_camera() -> void:
	_center_camera_on_map()