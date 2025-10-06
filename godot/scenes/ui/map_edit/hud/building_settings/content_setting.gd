extends PanelContainer

var building_id := -1

@onready var _content: LineEdit = %Content
@onready var _save_button: Button = %SaveButton
@onready var _close_button: Button = %CloseButton

func _ready() -> void:
	hide()
	_save_button.pressed.connect(_on_save_pressed)
	_close_button.pressed.connect(_close)

func open(target_building_id: int) -> void:
	building_id = target_building_id
	_update_from_backend()
	show()

func _close() -> void:
	hide()
	building_id = -1

func _on_save_pressed() -> void:
	if building_id < 0:
		return
	var rule := {}
	rule["pattern"] = _content.text
	EditorManager.set_filter_rules(building_id, rule)
	_close()

func _update_from_backend() -> void:
	var rule = EditorManager.get_filter_rule(building_id)
	var pattern = rule.get("pattern", "")
	_content.text = str(pattern)
	
func _unhandled_key_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		if visible:
			_on_save_pressed()
			get_viewport().set_input_as_handled()
