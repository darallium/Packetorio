extends Control

@onready var start_button: Button = %StartButton
@onready var stop_button: Button = %StopButton
#@onready var pause_button: Button = %PauseButton

func _ready() -> void:
	# ボタンのシグナルを接続
	if start_button:
		start_button.pressed.connect(_on_start_button_pressed)
	if stop_button:
		stop_button.pressed.connect(_on_stop_button_pressed)
	# PauseButtonは現在使用しないが、将来のために残す
	#if pause_button:
	#	pause_button.pressed.connect(_on_pause_button_pressed)

func _on_start_button_pressed() -> void:
	if EditorManager.map_controller == null:
		print("Error: map_controller is not initialized")
		return
	
	EditorManager.map_controller.start_heart_beat()
	EditorManager.start_game_timer()
	print("Simulation started")

func _on_stop_button_pressed() -> void:
	if EditorManager.map_controller == null:
		print("Error: map_controller is not initialized")
		return
	
	EditorManager.map_controller.stop_the_world()
	EditorManager.stop_game_timer()
	print("Simulation stopped")
