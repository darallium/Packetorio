extends "res://scenes/ui/sound_button.gd"

func _ready() -> void:
	if texts == "Default":
		texts = "やめておく"
	super._ready()

func on_pressed():
	super.on_pressed()
	get_tree().quit()
