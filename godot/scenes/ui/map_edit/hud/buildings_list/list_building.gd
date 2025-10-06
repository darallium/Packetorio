extends Button

@onready var icon_texture: TextureRect = $MainPanelContainer/ControlPanel/HBoxContainer/IconContainer/IconPanel/IconTexture
@onready var name_label: Label = $MainPanelContainer/ControlPanel/HBoxContainer/LabelContainer/VBoxContainer/NameLabel
@onready var cost_label: Label = $MainPanelContainer/ControlPanel/HBoxContainer/LabelContainer/VBoxContainer/CostLabel
@onready var control_panel: PanelContainer = $MainPanelContainer/ControlPanel
@onready var icon_container: MarginContainer = $MainPanelContainer/ControlPanel/HBoxContainer/IconContainer

var building_id: int = -1
var panel_style: StyleBoxFlat

const NORMAL_COLOR = Color("#353535")
const HOVER_COLOR = Color("#252525")

func _ready() -> void:
	panel_style = control_panel.get_theme_stylebox("panel").duplicate()
	control_panel.add_theme_stylebox_override("panel", panel_style)
	panel_style.bg_color = NORMAL_COLOR
	self.mouse_entered.connect(_on_mouse_entered)
	self.mouse_exited.connect(_on_mouse_exited)
	self.pressed.connect(_on_pressed)

func setup(building_data: Dictionary) -> void:
	if icon_texture and building_data.has("texture"):
		var base_texture: Texture2D = building_data["texture"]
		var icon_size: int
		
		if base_texture.resource_path == "res://assets/images/conve-Sheet.png":
			icon_size = 16
		else:
			var tex_width: int = base_texture.get_width()
			var tex_height: int = base_texture.get_height()
			icon_size = tex_width if tex_width < tex_height else tex_height
		var atlas_texture := AtlasTexture.new()
		atlas_texture.atlas = base_texture
		atlas_texture.region = Rect2(0, 0, icon_size, icon_size)
		icon_texture.texture = atlas_texture
		
		icon_texture.custom_minimum_size = Vector2(icon_size, icon_size)
	
	if name_label:
		name_label.text = building_data["name"]
	
	if cost_label:
		if building_data.has("cost"):
			cost_label.text = "Cost: " + str(building_data["cost"])
		else:
			cost_label.text = "Cost: N/A"
	
	self.building_id = building_data["id"]

func _on_pressed() -> void:
	EditorManager.select_building(building_id)

func _on_mouse_entered():
	panel_style.bg_color = HOVER_COLOR

func _on_mouse_exited():
	panel_style.bg_color = NORMAL_COLOR
