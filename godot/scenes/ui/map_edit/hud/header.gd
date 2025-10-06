extends CanvasLayer

@onready var cost_label: Label = $CostLabel
@onready var message_timer: Timer = $MessageTimer
@onready var timer_label: Label = $TimerLabel

var original_cost_text: String = ""
var last_cost: int = 0

func _ready():
	message_timer.timeout.connect(_on_message_timer_timeout)

func _process(delta: float) -> void:
	# タイマーを更新
	EditorManager.update_timer(delta)
	update_timer(EditorManager.get_remaining_time())
	
	if EditorManager.map_controller:
		var score = 0
		
		# RecycleBinのパケットを処理
		var recycle_bin_packets = EditorManager.map_controller.get_recyclebin_packets()
		for packet in recycle_bin_packets:
			var label = packet.get("label", 0) # Unknownは0
			if label == -1: # Incorrect
				score += 1
			elif label == 1: # Correct
				score -= 5
		
		# Datacenterのパケットを処理
		var datacenter_packets = EditorManager.map_controller.get_datacenter_packets()
		for packet in datacenter_packets:
			var label = packet.get("label", 0) # Unknownは0
			if label == -1: # Incorrect
				score -= 5
			elif label == 1: # Correct
				score += 5
		
		# コストの変動をスコアに反映
		var cost_diff = EditorManager.current_cost - last_cost
		score -= cost_diff
		last_cost = EditorManager.current_cost
		EditorManager.current_score = score
		update_score(score)

func show_message(text):
	$Message.text = text
	$Message.show()
	$MessageTimer.start()

func update_timer(time_seconds: float):
	var seconds = int(ceil(time_seconds))
	if timer_label:
		timer_label.text = "%d s" % seconds

func update_score(score):
	$ScoreLabel.text = "%04d" % score

func update_cost_preview(preview_cost: int):
	# 加算方式では常に正の値なので色の変更は不要
	var text = "Cost: %d" % [preview_cost]
	cost_label.text = text
	original_cost_text = text

func show_cost_message(message: String, duration: float = 2.0):
	if not message_timer.is_stopped():
		return
	original_cost_text = cost_label.text
	cost_label.text += "\n" + message
	message_timer.start(duration)
	
func _on_message_timer_timeout():
	cost_label.text = original_cost_text
