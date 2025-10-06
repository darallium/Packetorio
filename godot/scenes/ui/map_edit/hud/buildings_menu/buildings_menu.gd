extends Control

@onready var list_view: Control = %BuildingList

func _ready() -> void:
	$Button.button_down.connect(_on_button_button_down)
	$Button2.button_down.connect(_on_button_2_button_down)
	$Button3.button_down.connect(_on_button_3_button_down)

func _on_button_button_down() -> void:
	print("Button 1 (Basic Buildings) pressed")
	list_view.populate_list("basic")
	list_view.visible = true

func _on_button_2_button_down() -> void:
	print("Button 2 (Filters) pressed")
	list_view.populate_list("filters")
	list_view.visible = true

func _on_button_3_button_down() -> void:
	print("Button 3 pressed - Reserved for future use")
