extends Node

@onready var loader: PcapLoader = $PcapLoader 

func _ready():
	print("get ready")
	var traffic := loader.load_traffic("res://assets/http.cap")
	if traffic.is_empty():
		push_error("pcap missing")
		return
	print("traffic loaded")
	if traffic.packet_count() > 0:
		var f = traffic.get_packet(0)
		var src_ip = f.src_ip
		var src_port = f.src_port
		var timestamp = f.timestamp
		print("timestamp: ", timestamp)
		print("src ip: ", src_ip, " src port: ", src_port)
