extends Control

@onready var header = $Header
@onready var stage_label = $Header/Label
@onready var back_button = $BackButton

var reset = []
var _highlighted_stage_button: Button

func reset_nodes():
	var idx = 0
	for child in get_children():
		if child is Button and child.has_method("update_lines"):
			child.position = reset[idx]
			child.update_lines()
			if child.has_method("set_persistent_highlight"):
				child.set_persistent_highlight(false)
			idx += 1
			child.button_pressed = false
			if child != $stage1: # stage1 is the root of the tree and we want it enabled
				child.disabled = true

	_apply_global_selection()

func _ready():
	back_button.pressed.connect(func(): SceneManager.change_scene("res://scenes/ui/main_menu.tscn"))
	header.position.y = -header.size.y * 2
	var tween = create_tween()
	tween.set_trans(Tween.TRANS_CUBIC)
	tween.set_ease(Tween.EASE_OUT)
	tween.tween_property(header, "position:y", 0, 0.5)
	
	$stage1.neighbors = [$stage2]
	$stage2.neighbors = [$stage3]
	$stage3.neighbors = [$stage4]
	$stage4.neighbors = [$stage5]
	for child in get_children():
		if child is Button and child.has_method("update_lines"):
			reset.push_back(child.position)
			child.z_index = 10
			if child.has_signal("stage_hovered"):
				child.connect("stage_hovered", Callable(self, "_on_stage_hovered"))

	_apply_global_selection()
	


func _on_stage_hovered(stage_name: String, stage_button: Button) -> void:
	stage_label.text = stage_name
	_update_highlight(stage_button)


func _update_highlight(stage_button: Button) -> void:
	if is_instance_valid(_highlighted_stage_button) and _highlighted_stage_button != stage_button:
		if _highlighted_stage_button.has_method("set_persistent_highlight"):
			_highlighted_stage_button.set_persistent_highlight(false)
	if stage_button and stage_button.has_method("set_persistent_highlight"):
		stage_button.set_persistent_highlight(true)
		SceneManager.selected_stage_button_name = stage_button.name
	else:
		SceneManager.selected_stage_button_name = ""
	_highlighted_stage_button = stage_button


func _apply_global_selection() -> void:
	var target_button: Button = null
	if SceneManager.selected_stage_button_name != "":
		var candidate = get_node_or_null(SceneManager.selected_stage_button_name)
		if candidate is Button and candidate.has_method("set_persistent_highlight"):
			target_button = candidate
	if target_button == null and has_node("stage1"):
		target_button = $stage1
	if target_button:
		stage_label.text = _stage_button_display_name(target_button)
	else:
		stage_label.text = ""
	_update_highlight(target_button)


func _stage_button_display_name(stage_button: Button) -> String:
	if stage_button == null:
		return ""
	if stage_button.has_method("set_persistent_highlight"):
		var stage_data = stage_button.stage_data
		if stage_data is Dictionary and stage_data.has("meta") and stage_data["meta"].has("mapname"):
			return stage_data["meta"]["mapname"]
	return stage_button.name
