extends Button

var neighbors = []: set = set_neighbors
var lines = []
var neighbor_lines = []
var delay = 10
signal stage_hovered(stage_name: String, stage_button: Button)
signal stage_unhovered()

@export var background_normal: Texture2D = preload("res://assets/images/stages/stage_button_bg.png")
@export var background_disabled: Texture2D = preload("res://assets/images/stages/stage_button_bg_disabled.png")
@export var icon_texture: Texture2D

@onready var icon_rect: TextureRect = get_node_or_null("Icon")

var _background_rect: TextureRect

const STAGE_DATA = {
	"stage1": {
		"stage_data": "res://assets/stages/stage1_map.json",
		"packets_data": "res://assets/packets/stage1_packets.json",
	},
	"stage2": {
		"stage_data": "res://assets/stages/stage2_map.json",
		"packets_data": "res://assets/packets/stage2_packets.json",
	},
	"stage3": {
		"stage_data": "res://assets/stages/stage3_map.json",
		"packets_data": "res://assets/packets/stage3_packets.json",
	},
	"stage4": {
		"stage_data": "res://assets/stages/stage4_map.json",
		"packets_data": "res://assets/packets/stage4_packets.json",
	},
	"stage5": {
		"stage_data": "res://assets/stages/stage5_map.json",
		"packets_data": "res://assets/packets/stage5_packets.json",
	},
}

var stage_data
var packets_data
var _hover_stylebox: StyleBox
var _persistent_highlight := false
var _wants_persistent_highlight := false

func add_line(neighbor):
	var line_outline = Line2D.new()
	line_outline.width = 9
	line_outline.default_color = Color.BLACK
	line_outline.z_index = 1
	line_outline.add_point(position + size / 2)
	line_outline.add_point(neighbor.position + neighbor.size / 2)
	get_parent().add_child(line_outline)

	var line_fill = Line2D.new()
	line_fill.width = 3
	line_fill.default_color = Color.WHITE
	line_fill.z_index = 2
	line_fill.add_point(position + size / 2)
	line_fill.add_point(neighbor.position + neighbor.size / 2)
	get_parent().add_child(line_fill)

	var line_pair = { "fill": line_fill, "outline": line_outline }
	lines.push_back(line_pair)
	neighbor.neighbor_lines.push_back(line_pair)

func update_lines():
	for line_pair in lines:
		line_pair.fill.set_point_position(0, position + size / 2)
		line_pair.outline.set_point_position(0, position + size / 2)
	for line_pair in neighbor_lines:
		line_pair.fill.set_point_position(1, position + size / 2)
		line_pair.outline.set_point_position(1, position + size / 2)

func set_neighbors(new_neighbors):
	for neighbor in new_neighbors:
		if not neighbors.has(new_neighbors):
			add_line(neighbor)
	neighbors = new_neighbors

func _ready():
	if STAGE_DATA.has(name):
		var map_path = STAGE_DATA[name]["stage_data"]
		var packets_path = STAGE_DATA[name]["packets_data"]
		stage_data = JSON.parse_string(FileAccess.get_file_as_string(map_path))
		packets_data = JSON.parse_string(FileAccess.get_file_as_string(packets_path))
	else:
		stage_data = {}
		packets_data = {}
		if not Engine.is_editor_hint():
			push_warning("Stage data not found for '%s'" % name)
	
	if icon_rect and icon_texture:
		var tex_size: Vector2i = icon_texture.get_size()
		if tex_size == Vector2i(128, 32):
			var atlas_texture := AtlasTexture.new()
			atlas_texture.atlas = icon_texture
			atlas_texture.region = Rect2(0, 0, 32, 32)
			icon_rect.texture = atlas_texture
		elif tex_size == Vector2i(64, 16):
			var atlas_texture := AtlasTexture.new()
			atlas_texture.atlas = icon_texture
			atlas_texture.region = Rect2(0, 0, 16, 16)
			icon_rect.texture = atlas_texture
		else:
			icon_rect.texture = icon_texture
		icon_rect.z_index = 5
	_background_rect = get_node_or_null("Background")
	_background_rect.z_index = 4
	update_icon()
	connect("mouse_entered", Callable(self, "_on_mouse_entered"))
	connect("mouse_exited", Callable(self, "_on_mouse_exited"))
	_hover_stylebox = get_theme_stylebox("hover")

func _notification(what):
	if what == NOTIFICATION_DISABLED or what == NOTIFICATION_ENABLED:
		update_icon()
		_apply_persistent_highlight()

func update_icon():
	if _background_rect == null:
		_background_rect = get_node_or_null("Background")
	if _background_rect == null:
		return
	if disabled:
		if icon_rect:
			icon_rect.modulate = Color(1, 1, 1, 0.4)
		if background_disabled:
			_background_rect.texture = background_disabled
	else:
		if icon_rect:
			icon_rect.modulate = Color(1, 1, 1, 1)
		if background_normal:
			_background_rect.texture = background_normal


func _on_Button_toggled(button_pressed):
	if pressed and button_pressed:
		for neighbor in neighbors:
			neighbor.disabled = false
			neighbor.update_icon()


func _process(_delta):
	pass
	#if Input.is_action_pressed("mouse_left") and is_hovered():
		#delay -= 1
		#if delay < 0:
			#position = get_global_mouse_position() - size / 2 # Center mouse
			#update_lines()
	#if Input.is_action_just_released("mouse_left"):
		#delay = 10

func _on_mouse_entered():
	if disabled:
		return
	var label_text = name
	if stage_data is Dictionary and stage_data.has("meta") and stage_data["meta"].has("mapname"):
		label_text = stage_data["meta"]["mapname"]
	_emit_hover(label_text)

func _on_mouse_exited():
	emit_signal("stage_unhovered")


func _emit_hover(stage_name: String) -> void:
	emit_signal("stage_hovered", stage_name, self)
	set_persistent_highlight(true)


func _on_pressed():
	print("stage transition: " + name)
	
	if STAGE_DATA.has(name):
		var map_path = STAGE_DATA[name]["stage_data"]
		var packets_path = STAGE_DATA[name]["packets_data"]
		print("Selected map path: ", map_path)
		print("Selected packets path: ", packets_path)
		
		# EditorManagerにマップパスとパケットパスを設定
		EditorManager.selected_map_path = map_path
		EditorManager.selected_packets_path = packets_path
		EditorManager.reset_editor_state()
		
		# editor_mainシーンに遷移
		SceneManager.change_scene("res://scenes/ui/map_edit/editor_main.tscn")
	else:
		push_warning("Stage data not found for: " + name)


func set_persistent_highlight(enabled: bool) -> void:
	_wants_persistent_highlight = enabled
	_apply_persistent_highlight()


func _apply_persistent_highlight() -> void:
	var should_highlight = _wants_persistent_highlight and not disabled
	if should_highlight == _persistent_highlight:
		return
	_persistent_highlight = should_highlight
	if _persistent_highlight:
		if _hover_stylebox:
			add_theme_stylebox_override("normal", _hover_stylebox)
			add_theme_stylebox_override("pressed", _hover_stylebox)
	else:
		if has_theme_stylebox_override("normal"):
			remove_theme_stylebox_override("normal")
		if has_theme_stylebox_override("pressed"):
			remove_theme_stylebox_override("pressed")
