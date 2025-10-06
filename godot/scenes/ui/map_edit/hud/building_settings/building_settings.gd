extends PanelContainer

var building_id := -1
var building_type_id := -1

@onready var title_label: Label = %TitleLabel
@onready var protocol_row: VBoxContainer = %ProtocolRow
@onready var direction_row: VBoxContainer = %DirectionRow
@onready var port_row: VBoxContainer = %PortRow
@onready var ip_row: VBoxContainer = %IPRow

@onready var protocol_option: OptionButton = %ProtocolOption
@onready var direction_option: OptionButton = %DirectionOption
@onready var port_input: LineEdit = %PortInput
@onready var ip_octet1: LineEdit = %Octet1
@onready var ip_octet2: LineEdit = %Octet2
@onready var ip_octet3: LineEdit = %Octet3
@onready var ip_octet4: LineEdit = %Octet4

@onready var save_button: Button = %SaveButton
@onready var close_button: Button = %CloseButton

const BUILDING_INFO = {
	10: {"name": "IP Filter"},
	11: {"name": "Port Filter"},
	13: {"name": "Protocol Filter"},
}

func _ready() -> void:
	hide()
	_setup_options()

	# Save/Closeボタン接続
	save_button.pressed.connect(_on_save_pressed)
	close_button.pressed.connect(_close)

	# 数字入力制限 (Port + IP)
	for input in [port_input, ip_octet1, ip_octet2, ip_octet3, ip_octet4]:
		input.text_changed.connect(_on_number_input_changed.bind(input))

	# IP未入力チェックで赤く光る
	for input in [ip_octet1, ip_octet2, ip_octet3, ip_octet4]:
		input.text_changed.connect(_on_ip_octet_changed)

	# IPオクテット自動移動
	var ip_inputs = [ip_octet1, ip_octet2, ip_octet3, ip_octet4]
	for i in range(ip_inputs.size()):
		ip_inputs[i].connect("gui_input", Callable(self, "_on_ip_octet_gui_input").bind(i, ip_inputs))


func _setup_options() -> void:
	protocol_option.add_item("tcp")
	protocol_option.add_item("udp")
	protocol_option.add_item("unknown")
	direction_option.add_item("source")
	direction_option.add_item("destination")


func open(target_building_id: int, target_building_type_id: int) -> void:
	building_id = target_building_id
	building_type_id = target_building_type_id

	if not BUILDING_INFO.has(building_type_id):
		_close()
		return
	
	_reset_ip_octet_styles()  
	_update_ui_visibility()
	_update_from_backend()
	show()


func _close() -> void:
	hide()
	building_id = -1
	building_type_id = -1


func _update_ui_visibility() -> void:
	protocol_row.hide()
	direction_row.hide()
	port_row.hide()
	ip_row.hide()

	title_label.text = BUILDING_INFO[building_type_id]["name"] + " Settings"

	match building_type_id:
		10:
			ip_row.show()
			direction_row.show()
		11:
			port_row.show()
			direction_row.show()
		13:
			protocol_row.show()


func _update_from_backend() -> void:
	var rule = EditorManager.get_filter_rule(building_id)

	match building_type_id:
		10:
			var ip = rule.get("target_ip", null)
			if ip != null and not ip.is_empty():
				var ip_parts = ip.split(".")
				if ip_parts.size() == 4:
					ip_octet1.text = ip_parts[0]
					ip_octet2.text = ip_parts[1]
					ip_octet3.text = ip_parts[2]
					ip_octet4.text = ip_parts[3]
			else:
				ip_octet1.text = ""
				ip_octet2.text = ""
				ip_octet3.text = ""
				ip_octet4.text = ""

			var direction = rule.get("direction", "source")
			for i in range(direction_option.item_count):
				if direction_option.get_item_text(i) == direction:
					direction_option.select(i)
					break

		11:
			var port = rule.get("target_port", null)
			port_input.text = str(port) if port != null else ""

			var direction = rule.get("direction", "source")
			for i in range(direction_option.item_count):
				if direction_option.get_item_text(i) == direction:
					direction_option.select(i)
					break

		13:
			var protocol = rule.get("protocol", "tcp")
			for i in range(protocol_option.item_count):
				if protocol_option.get_item_text(i) == protocol:
					protocol_option.select(i)
					break


func _on_save_pressed() -> void:
	if building_id < 0:
		return

	var rule := {}

	match building_type_id:
		10:
			var ip_address = "%s.%s.%s.%s" % [ip_octet1.text, ip_octet2.text, ip_octet3.text, ip_octet4.text]
			rule["target_ip"] = ip_address if ip_address != "..." else ""
			rule["direction"] = direction_option.get_item_text(direction_option.selected)
		11:
			if not port_input.text.is_empty() and port_input.text.is_valid_int():
				rule["target_port"] = int(port_input.text)
			else:
				rule["target_port"] = 0
			rule["direction"] = direction_option.get_item_text(direction_option.selected)
		13:
			rule["protocol"] = protocol_option.get_item_text(protocol_option.selected)

	EditorManager.set_filter_rules(building_id, rule)
	_close()


func _on_number_input_changed(new_text: String, line_edit: LineEdit) -> void:
	var filtered := ""
	for c in new_text:
		if c >= "0" and c <= "9":
			filtered += c
	if filtered != new_text:
		line_edit.text = filtered


func _on_ip_octet_changed(_new_text: String) -> void:
	for input in [ip_octet1, ip_octet2, ip_octet3, ip_octet4]:
		if input.text.is_empty():
			var style := StyleBoxFlat.new()
			style.border_color = Color(1, 0.6, 0.6)
			style.set_border_width_all(2)
			style.set_corner_radius_all(2)
			input.add_theme_stylebox_override("normal", style)
		else:
			input.remove_theme_stylebox_override("normal")

func _on_ip_octet_gui_input(event: InputEvent, index: int, inputs: Array) -> void:
	if event is InputEventKey and event.pressed and event.unicode != 0:
		var input = inputs[index]
		if input.text.length() >= 3:
			if index < inputs.size() - 1:
				inputs[index + 1].grab_focus()
			else:
				save_button.grab_focus()

func _unhandled_key_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		if visible:
			_on_save_pressed()
			get_viewport().set_input_as_handled()
			
func _reset_ip_octet_styles() -> void:
	for input in [ip_octet1, ip_octet2, ip_octet3, ip_octet4]:
		input.remove_theme_stylebox_override("normal")
