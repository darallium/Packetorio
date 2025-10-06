extends Control

const ListBuildingScene = preload("res://scenes/ui/map_edit/hud/buildings_list/list_building.tscn")

const BUILDING_DATA = {
	"basic": [
		{"name": "Internet", "id": 0, "cost": 50, "size": Vector2i(2, 2), "texture": preload("res://assets/images/internet.png")},
		{"name": "Datacenter", "id": 1, "cost": 100, "size": Vector2i(2, 2), "texture": preload("res://assets/images/datacenter.png")},
		{"name": "Conveyor", "id": 2, "cost": 1, "size": Vector2i(1, 1), "texture": preload("res://assets/images/conve-Sheet.png")},
		{"name": "Junction", "id": 15, "cost": 5, "size": Vector2i(1, 1), "texture": preload("res://assets/images/router.png")},
		{"name": "Recycle Bin", "id": 16, "cost": 10, "size": Vector2i(1, 1), "texture": preload("res://assets/images/recycler.png")},
	],
	"filters": [
		{"name": "IP Filter", "id": 10, "cost": 20, "size": Vector2i(1, 1), "texture": preload("res://assets/images/filters/ip-filter.png")},
		{"name": "Port Filter", "id": 11, "cost": 20, "size": Vector2i(1, 1), "texture": preload("res://assets/images/filters/port-filter.png")},
		{"name": "Length Filter", "id": 12, "cost": 15, "size": Vector2i(1, 1), "texture": preload("res://assets/images/filters/length-filter.png")},
		{"name": "Protocol Filter", "id": 13, "cost": 15, "size": Vector2i(1, 1), "texture": preload("res://assets/images/filters/protocol-filter.png")},
		{"name": "Content Filter", "id": 14, "cost": 25, "size": Vector2i(1, 1), "texture": preload("res://assets/images/filters/content-filter.png")},
	]
}

@onready var hbox_container: HBoxContainer = $ScrollContainer/MarginContainer/HBoxContainer

func _ready() -> void:
	self.visible = false

func populate_list(genre_key: String) -> void:
	for child in hbox_container.get_children():
		child.queue_free()

	if not BUILDING_DATA.has(genre_key):
		print("Error: Genre '", genre_key, "' not found in BUILDING_DATA.")
		return
	
	var buildings = BUILDING_DATA[genre_key]

	for building_data in buildings:
		if building_data["name"] == "Internet" or building_data["name"] == "Datacenter":
			continue  # Skip these buildings for now
		var instance = ListBuildingScene.instantiate()
		hbox_container.add_child(instance)
		instance.setup(building_data)
