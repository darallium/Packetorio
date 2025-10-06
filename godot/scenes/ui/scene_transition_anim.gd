extends AnimationPlayer

func transion_start():
	#play("transition_enter")
	#await get_tree().create_timer(0.5).timeout
	play("transition_exit")
	
