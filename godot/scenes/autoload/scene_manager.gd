extends Node

# SubViewportへの参照
var game_viewport: SubViewport = null

# SceneTransitionShaderへの参照
var scene_transition_func: Callable

# Stage selection state (keeps last highlighted button between scenes)
var selected_stage_button_name := ""

# シーンのロード
func change_scene(path: String):
	print(path)
	if not game_viewport:
		
		printerr("Game Viewport is not set.")
		get_tree().change_scene_to_packed(load(path))
		return
	
	if scene_transition_func:
		scene_transition_func.call_deferred()
	
	if game_viewport.get_child_count() > 0:
		game_viewport.get_child(0).queue_free()
	
	var next_scene = load(path).instantiate()
	game_viewport.add_child(next_scene)
