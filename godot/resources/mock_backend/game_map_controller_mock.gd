extends Node # Nodeを継承するように変更
# game_map_controller_mock.gd

class UnitDTO:
	var unit_type: int
	var rotation: int
	func _init(_unit_type, _rotation):
		unit_type = _unit_type
		rotation = _rotation

class UnitInfo:
	var unit_type: int
	var rotation: int
	func _init(_unit_type, _rotation):
		unit_type = _unit_type
		rotation = _rotation

static var _units: Dictionary = {} # Dictionary to store unit info by coordinate (Vector2i as key) # staticを追加

static func _init(): # staticを追加
	print("GameMapControllerMock initialized.")
	print("GameMapControllerMock.new called with width: %s, height: %s" % [100, 100]) # 例として100x100
	# _units.clear() # 必要に応じてマップを初期化
	
static func place_unit(cord: Vector2i, unit_type: int, rotation: int): # staticを追加
	print("GameMapControllerMock.place_unit called: cord=%s, unit_type=%s, rotation=%s" % [cord, unit_type, rotation])
	_units[cord] = UnitInfo.new(unit_type, rotation)

static func remove_unit(cord: Vector2i): # staticを追加
	print("GameMapControllerMock.remove_unit called: cord=%s" % cord)
	if _units.has(cord):
		_units.erase(cord)

static func get_unit_info(cord: Vector2i) -> UnitInfo: # staticを追加
	print("GameMapControllerMock.get_unit_info called: cord=%s" % cord)
	return _units.get(cord, null)

static func get_all_unit_coordinates() -> Array: # staticを追加
	print("GameMapControllerMock.get_all_unit_coordinates called.")
	return _units.keys()
