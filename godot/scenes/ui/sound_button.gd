@tool
extends Button
@onready var random_audio_stream_player = %RandomAudioStreamPlayer
@onready var label = %Label

# インスペクターに "Text" という項目を追加し、文字列 (String) を設定できるようにする
@export var texts: String = "Default":
	set(value):
		texts = value
		# このスクリプトがシーンツリー内にあるかを確認してから処理を行う
		var label = %Label
		if label:
			label.text = value

# --- ここまで追記 ---

# _readyは、ゲーム実行時に一度だけ呼ばれる関数
func _ready() -> void:
	# 起動時にエクスポートされた値で初期化する
	%Label.text = texts
	mouse_entered.connect(on_mouse_entered)
	pressed.connect(on_pressed)


func play_sound():
	if (!disabled):
		random_audio_stream_player.play_random()


func on_pressed():
	play_sound()


func on_mouse_entered():
	play_sound()
