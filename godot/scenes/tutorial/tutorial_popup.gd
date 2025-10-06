extends Control

## チュートリアル用ポップアップウィンドウ
## マップ読み込み時などに表示されるメッセージボックス

# ノード参照（固有名でアクセス）
@onready var background_overlay: ColorRect = %BackgroundOverlay
@onready var popup_panel: PanelContainer = %PopupPanel
@onready var title_label: Label = %TitleLabel
@onready var message_label: Label = %MessageLabel
@onready var ok_button: Button = %OKButton

# ポップアップが閉じられた後に呼ばれるコールバック関数（オプション）
var on_closed: Callable = Callable()

func _ready() -> void:
	# 初期状態で非表示
	visible = false
	
	# OKボタンのシグナル接続
	if ok_button:
		ok_button.pressed.connect(_on_ok_button_pressed)
	
	# 背景オーバーレイのクリックは無効化（ポップアップ外クリックで閉じない）
	if background_overlay:
		background_overlay.mouse_filter = Control.MOUSE_FILTER_STOP

## ポップアップを表示する
## @param title: タイトル文字列
## @param message: メッセージ本文
## @param position: 表示位置（デフォルト: Vector2.ZERO = 画面中央）
## @param size: ポップアップサイズ（デフォルト: Vector2.ZERO = 自動サイズ）
## @param closed_callback: 閉じられた時のコールバック（オプション）
func show_popup(
	title: String,
	message: String,
	position: Vector2 = Vector2.ZERO,
	size: Vector2 = Vector2.ZERO,
	closed_callback: Callable = Callable()
) -> void:
	# テキスト設定
	if title_label:
		title_label.text = title
	
	if message_label:
		message_label.text = message
	
	# コールバック設定
	on_closed = closed_callback
	
	# サイズ設定（0の場合は自動サイズ）
	if size != Vector2.ZERO and popup_panel:
		popup_panel.custom_minimum_size = size
	
	# 表示
	visible = true
	
	# 位置調整（次フレームで実行、サイズが確定してから）
	await get_tree().process_frame
	_update_position(position)
	
	print("Tutorial popup shown: ", title)

## ポップアップの位置を更新
## @param position: 指定位置（Vector2.ZEROの場合は画面中央）
func _update_position(position: Vector2) -> void:
	if not popup_panel:
		return
	
	if position == Vector2.ZERO:
		# 画面中央に配置
		var viewport_size = get_viewport_rect().size
		var panel_size = popup_panel.size
		popup_panel.position = (viewport_size - panel_size) / 2.0
	else:
		# 指定位置に配置
		popup_panel.position = position

## OKボタンが押された時の処理
func _on_ok_button_pressed() -> void:
	print("Tutorial popup closed")
	
	# コールバック実行
	if on_closed.is_valid():
		on_closed.call()
	
	# ポップアップを閉じる
	close_popup()

## ポップアップを閉じる
func close_popup() -> void:
	visible = false
	# 必要に応じてqueuefreeも可能
	# queue_free()
