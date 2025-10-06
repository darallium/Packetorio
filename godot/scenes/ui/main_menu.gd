extends CanvasLayer

@onready var start_button = %StartButton
@onready var options_button = %OptionsButton
@onready var quit_button = %QuitButton

var options_menu_scene: PackedScene = preload("res://scenes/ui/options_menu.tscn")

# Called when the node enters the scene tree for the first time.
func _ready():
	start_button.pressed.connect(func(): SceneManager.change_scene("res://scenes/ui/stage/Tree.tscn"))
	options_button.pressed.connect(on_options_pressed)
	quit_button.pressed.connect(func(): get_tree().quit())


func on_options_pressed():
	var options_menu_instance = options_menu_scene.instantiate()
	options_menu_instance.back_pressed.connect(on_options_back_pressed.bind(options_menu_instance))
	add_child(options_menu_instance)

func on_options_back_pressed(options_menu: OptionsMenu):
	options_menu.queue_free()
