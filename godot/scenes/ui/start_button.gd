extends TextureButton

# ボタンの初期位置を保存する変数
var initial_position: Vector2
# 現在実行中のTweenを管理する変数
var active_tween: Tween

func _ready():
	# ゲーム開始時にボタンの初期位置を保存
	initial_position = self.position
	# mouse_enteredとmouse_exitedシグナルをコードで接続
	mouse_entered.connect(_on_mouse_entered)
	mouse_exited.connect(_on_mouse_exited)

# マウスカーソルがボタンの上に乗ったときに呼び出される関数
func _on_mouse_entered():
	# 既存のアニメーションがあれば中断して破棄する
	if active_tween:
		active_tween.kill()

	# 新しいTweenを作成
	active_tween = create_tween()
	# イージングとトランジションのタイプを設定
	# EASE_OUT: ゆっくり終わる
	# TRANS_EXPO: 指数関数的に変化（急激に始まり、変化が大きい）
	active_tween.set_ease(Tween.EASE_OUT).set_trans(Tween.TRANS_EXPO)
	
	# "position"プロパティを初期位置から右に10px移動した位置まで0.2秒かけてアニメーションさせる
	active_tween.tween_property(self, "position", initial_position + Vector2(10, 0), 0.2)

# マウスカーソルがボタンから離れたときに呼び出される関数
func _on_mouse_exited():
	# 既存のアニメーションがあれば中断して破棄する
	if active_tween:
		active_tween.kill()

	# 新しいTweenを作成
	active_tween = create_tween()
	# こちらも同じイージングを設定して、動きの感覚を統一します
	active_tween.set_ease(Tween.EASE_OUT).set_trans(Tween.TRANS_EXPO)

	# "position"プロパティを初期位置まで0.2秒かけてアニメーションさせる
	active_tween.tween_property(self, "position", initial_position, 0.2)
