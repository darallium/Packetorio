extends Node2D

var packet_size: Vector2 = Vector2(16, 16)  # パケットの表示サイズ

var last_correct_count: int = 0
var last_incorrect_count: int = 0
var last_unknown_count: int = 0

@onready var correct_multimesh: MultiMeshInstance2D = %MultimeshCorrectPackets
@onready var incorrect_multimesh: MultiMeshInstance2D = %MultimeshIncorrectPackets
@onready var unknown_multimesh: MultiMeshInstance2D = %MultimeshUnknownPackets

func _ready():
	# 各MultiMeshの初期設定
	for mesh in [correct_multimesh, incorrect_multimesh, unknown_multimesh]:
		if mesh and mesh.multimesh:
			mesh.multimesh.transform_format = MultiMesh.TRANSFORM_2D
			mesh.multimesh.instance_count = 0
	print("MultiMesh instances initialized for packets")

func _process(_delta: float) -> void:
	if not EditorManager.map_controller:
		return
	
	# バックエンドからパケット位置を取得
	var packet_positions = EditorManager.map_controller.get_all_packet_positions()
	
	if not packet_positions:
		clear_packets()
		return
	
	# パケットをラベルごとに分類
	var correct_packets = []
	var incorrect_packets = []
	var unknown_packets = []
	
	for packet in packet_positions:
		
		var label = packet.get("label", 0)  # デフォルトはUnknown (0)
		match label:
			1:  # Correct
				correct_packets.append(packet)
			-1:  # Incorrect
				incorrect_packets.append(packet)
			_:  # Unknown (0)
				unknown_packets.append(packet)
	
	# 各カテゴリのパケットを更新
	update_packet_instances(correct_multimesh, correct_packets, last_correct_count)
	update_packet_instances(incorrect_multimesh, incorrect_packets, last_incorrect_count)
	update_packet_instances(unknown_multimesh, unknown_packets, last_unknown_count)
	
	# カウントを更新
	last_correct_count = correct_packets.size()
	last_incorrect_count = incorrect_packets.size()
	last_unknown_count = unknown_packets.size()

func update_packet_instances(mesh_instance: MultiMeshInstance2D, packets: Array, last_count: int) -> void:
	if not mesh_instance or not mesh_instance.multimesh:
		return
		
	var current_count = packets.size()
	
	# パケット数が変わった場合はインスタンス数を更新
	if current_count != last_count:
		mesh_instance.multimesh.instance_count = current_count
	
	# 各パケットの位置を設定
	for i in range(current_count):
		var packet_data = packets[i]
		var position = packet_data["position"]
		
		# Transform2Dを作成（位置とサイズ）
		var transform = Transform2D()
		transform = transform.scaled(packet_size / 16.0)
		transform.origin = position
		
		mesh_instance.multimesh.set_instance_transform_2d(i, transform)

func clear_packets() -> void:
	for mesh in [correct_multimesh, incorrect_multimesh, unknown_multimesh]:
		if mesh and mesh.multimesh:
			mesh.multimesh.instance_count = 0
	
	last_correct_count = 0
	last_incorrect_count = 0
	last_unknown_count = 0
