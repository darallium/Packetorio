extends Control

@onready var game_viewport = %RendererViewport
@onready var scene_transition_anim = %SceneTransitionAnim

# 最初のシーンをロードする
func _ready():
	SceneManager.game_viewport = game_viewport
	SceneManager.change_scene("res://scenes/ui/main_menu.tscn")
	SceneManager.scene_transition_func = scene_transition_anim.transion_start
