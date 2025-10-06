extends MarginContainer

@onready var bg_texture_rect: TextureRect = $BG/Clipping/BGTextureRect
@onready var fg_texture_button: TextureButton = $FG/Clipping/FGTextureButton
@onready var fg_texture_button_pressed_texture = fg_texture_button.texture_pressed
@onready var fg_texture_button_normal_texture = fg_texture_button.texture_normal

func _ready() -> void:
	bg_texture_rect.texture = fg_texture_button_normal_texture

func _on_texture_button_2_button_down() -> void:
	_update_bg_texture_pressed()

func _on_texture_button_2_button_up() -> void:
	_update_bg_texture_normal()

func _update_bg_texture_pressed() -> void:
	bg_texture_rect.texture = fg_texture_button_pressed_texture

func _update_bg_texture_normal() -> void:
	bg_texture_rect.texture = fg_texture_button_normal_texture
