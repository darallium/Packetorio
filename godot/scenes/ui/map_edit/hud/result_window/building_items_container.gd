extends Control

const BUILDING_ITEM_SCENE = preload("res://scenes/ui/map_edit/hud/result_window/building_item.tscn")

const DATACENTER_TYPE = 1
const RECYCLE_BIN_TYPE = 9

@onready var datacenter_list: VBoxContainer = $HBoxContainer/DatacenterColumn/DatacenterScrollContainer/DatacenterList
@onready var recyclebin_list: VBoxContainer = $HBoxContainer/RecycleBinColumn/RecycleBinScrollContainer/RecycleBinList
@onready var datacenter_label: Label = $HBoxContainer/DatacenterColumn/DatacenterLabel
@onready var recyclebin_label: Label = $HBoxContainer/RecycleBinColumn/RecycleBinLabel

# アイコンのテクスチャ（後でユーザーがGodotEditorで設定）
@export var datacenter_icon: Texture2D
@export var recyclebin_icon: Texture2D

# Building一覧をセットアップ
func setup_buildings(buildings_data: Dictionary, result_window) -> void:
	# 既存のアイテムをクリア
	_clear_lists()
	
	var datacenter_count = 0
	var recyclebin_count = 0
	
	# building_idごとにbuilding_itemを生成
	for b_id in buildings_data.keys():
		var data = buildings_data[b_id]
		var b_type = data["building_type"]
		var position = data["position"]
		var packets = data["packets"]
		
		# building_itemを作成
		var item = BUILDING_ITEM_SCENE.instantiate()
		var icon = datacenter_icon if b_type == DATACENTER_TYPE else recyclebin_icon
		item.setup(b_id, b_type, position, packets, icon, result_window)
		
		# 適切なリストに追加
		if b_type == DATACENTER_TYPE:
			datacenter_list.add_child(item)
			datacenter_count += 1
		elif b_type == RECYCLE_BIN_TYPE:
			recyclebin_list.add_child(item)
			recyclebin_count += 1
	
	# ヘッダーラベルを更新（数を表示）
	if datacenter_label:
		datacenter_label.text = "Datacenter (%d)" % datacenter_count
		datacenter_label.visible = datacenter_count > 0
	
	if recyclebin_label:
		recyclebin_label.text = "Recycle Bin (%d)" % recyclebin_count
		recyclebin_label.visible = recyclebin_count > 0

# リストをクリア
func _clear_lists() -> void:
	if datacenter_list:
		for child in datacenter_list.get_children():
			child.queue_free()
	
	if recyclebin_list:
		for child in recyclebin_list.get_children():
			child.queue_free()

func _ready() -> void:
	# アイコンテクスチャの初期化（ユーザーがGodotEditorで設定する）
	pass
