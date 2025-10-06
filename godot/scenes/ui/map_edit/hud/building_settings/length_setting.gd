extends PanelContainer

var building_id := -1

@onready var _threshold: LineEdit = %Threshold
@onready var _operator: OptionButton = %Operator
@onready var _save_button: Button = %SaveButton
@onready var _close_button: Button = %CloseButton

const _DIRECTIONS := {
	0: "exact",
	1: "less_than",
	2: "greater_than",
}

func _ready() -> void:
	hide()
	_save_button.pressed.connect(_on_save_pressed)
	_close_button.pressed.connect(_close)
	_threshold.text_changed.connect(_on_threshold_changed)

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
	rule["threshold"] = int(_threshold.text)
	rule["direction"] = _DIRECTIONS.get(_operator.selected, "exact")
	EditorManager.set_filter_rules(building_id, rule)
	_close()

func _update_from_backend() -> void:
	var rule = EditorManager.get_filter_rule(building_id)
	_threshold.text = str(rule.get("threshold", ""))
	var dir = String(rule.get("direction", "exact"))
	var target_index = 0
	for index in _DIRECTIONS.keys():
		if _DIRECTIONS[index] == dir:
			target_index = index
			break
	_operator.selected = target_index

func _on_threshold_changed(new_text: String) -> void:
	var filtered := ""
	for c in new_text:
		if c >= "0" and c <= "9":
			filtered += c
	if filtered != new_text:
		_threshold.text = filtered

func _unhandled_key_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		if visible:
			_on_save_pressed()
			get_viewport().set_input_as_handled()
